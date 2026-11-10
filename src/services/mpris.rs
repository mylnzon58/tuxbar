//! MPRIS D-Bus client for discovering and controlling media players
//! Works with Spotify, Chrome, Firefox, VLC, and any MPRIS compliant player.

use zbus::blocking::Connection;
use zbus::zvariant::OwnedValue;

#[derive(Debug, Clone, Default)]
pub struct TrackInfo {
    pub player_name: String,
    pub title: String,
    pub artist: String,
    pub is_playing: bool,
    pub position_secs: f32,
    pub length_secs: f32,
}

pub struct MprisClient {
    conn: Option<Connection>,
    cached_info: TrackInfo,
    active_bus_name: Option<String>,
}

impl MprisClient {
    #[must_use]
    pub fn new() -> Self {
        let conn = Connection::session().ok().or_else(|| {
            let bus_path = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok().or_else(|| {
                if std::path::Path::new("/run/user/1000/bus").exists() {
                    Some("unix:path=/run/user/1000/bus".to_string())
                } else {
                    None
                }
            })?;
            zbus::blocking::connection::Builder::address(bus_path.as_str()).ok()?.build().ok()
        });

        Self {
            conn,
            cached_info: TrackInfo::default(),
            active_bus_name: None,
        }
    }

    pub fn update(&mut self) -> &TrackInfo {
        if self.conn.is_none() {
            let fresh = Self::new();
            if fresh.conn.is_some() {
                self.conn = fresh.conn;
            } else {
                self.cached_info = TrackInfo::default();
                return &self.cached_info;
            }
        }

        let Some(conn) = &self.conn else {
            return &self.cached_info;
        };

        // 1. Discover active players (prioritizing actively Playing players)
        let mut target_bus = self.active_bus_name.clone();

        if let Ok(reply) = conn.call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "ListNames",
            &(),
        ) {
            if let Ok(names) = reply.body().deserialize::<Vec<String>>() {
                let mpris_players: Vec<String> = names
                    .into_iter()
                    .filter(|n| n.starts_with("org.mpris.MediaPlayer2."))
                    .collect();

                // If current target no longer exists on D-Bus, reset it
                if let Some(ref curr) = target_bus {
                    if !mpris_players.contains(curr) {
                        target_bus = None;
                    }
                }

                // Check if any player is actively Playing
                let mut found_playing = None;
                for player in &mpris_players {
                    if let Ok(reply) = conn.call_method(
                        Some(player.as_str()),
                        "/org/mpris/MediaPlayer2",
                        Some("org.freedesktop.DBus.Properties"),
                        "Get",
                        &("org.mpris.MediaPlayer2.Player", "PlaybackStatus"),
                    ) {
                        if let Ok(val) = reply.body().deserialize::<OwnedValue>() {
                            let s = format!("{val:?}");
                            if s.contains("Playing") {
                                found_playing = Some(player.clone());
                                break;
                            }
                        }
                    }
                }

                if let Some(playing_bus) = found_playing {
                    target_bus = Some(playing_bus);
                } else if target_bus.is_none() && !mpris_players.is_empty() {
                    target_bus = Some(mpris_players[0].clone());
                }
            }
        }

        self.active_bus_name = target_bus;

        let Some(bus) = &self.active_bus_name else {
            self.cached_info = TrackInfo::default();
            return &self.cached_info;
        };

        // 2. Query PlaybackStatus
        let status_res = conn.call_method(
            Some(bus.as_str()),
            "/org/mpris/MediaPlayer2",
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.mpris.MediaPlayer2.Player", "PlaybackStatus"),
        );

        let is_playing = match status_res {
            Ok(reply) => {
                if let Ok(val) = reply.body().deserialize::<OwnedValue>() {
                    let s = format!("{val:?}");
                    s.contains("Playing")
                } else {
                    false
                }
            }
            Err(_) => {
                self.active_bus_name = None;
                self.cached_info = TrackInfo::default();
                return &self.cached_info;
            }
        };

        // 3. Query Metadata
        let mut title = String::new();
        let mut artist = String::new();
        let mut length_secs = 0.0f32;

        if let Ok(meta_reply) = conn.call_method(
            Some(bus.as_str()),
            "/org/mpris/MediaPlayer2",
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.mpris.MediaPlayer2.Player", "Metadata"),
        ) {
            if let Ok(val) = meta_reply.body().deserialize::<OwnedValue>() {
                if let zbus::zvariant::Value::Dict(dict) = val.into() {
                    for (k, v) in dict.iter() {
                        let key_str = format!("{k:?}");
                        if key_str.contains("xesam:title") {
                            title = extract_inner_str(v);
                        } else if key_str.contains("xesam:artist") {
                            artist = extract_inner_str(v);
                        } else if key_str.contains("mpris:length") {
                            length_secs = extract_inner_i64(v) as f32 / 1_000_000.0;
                        }
                    }
                }
            }
        }

        // 4. Query Position
        let mut position_secs = 0.0f32;
        if let Ok(pos_reply) = conn.call_method(
            Some(bus.as_str()),
            "/org/mpris/MediaPlayer2",
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.mpris.MediaPlayer2.Player", "Position"),
        ) {
            if let Ok(val) = pos_reply.body().deserialize::<OwnedValue>() {
                let pos = extract_i64(&val);
                if pos > 0 {
                    position_secs = (pos as f32) / 1_000_000.0;
                }
            }
        }

        let player_clean = bus.strip_prefix("org.mpris.MediaPlayer2.").unwrap_or(bus).to_string();

        self.cached_info = TrackInfo {
            player_name: player_clean,
            title: if title.is_empty() { "Audio Playing".to_string() } else { title },
            artist,
            is_playing,
            position_secs,
            length_secs,
        };

        &self.cached_info
    }

    pub fn play_pause(&self) {
        if let Some(conn) = &self.conn {
            let mut sent = false;
            if let Some(bus) = &self.active_bus_name {
                let _ = conn.call_method(
                    Some(bus.as_str()),
                    "/org/mpris/MediaPlayer2",
                    Some("org.mpris.MediaPlayer2.Player"),
                    "PlayPause",
                    &(),
                );
                sent = true;
            }
            if !sent {
                self.broadcast_method("PlayPause");
            }
        }
    }

    pub fn next(&self) {
        if let Some(conn) = &self.conn {
            if let Some(bus) = &self.active_bus_name {
                let _ = conn.call_method(
                    Some(bus.as_str()),
                    "/org/mpris/MediaPlayer2",
                    Some("org.mpris.MediaPlayer2.Player"),
                    "Next",
                    &(),
                );
            }
            self.broadcast_method("Next");
        }
    }

    pub fn previous(&self) {
        if let Some(conn) = &self.conn {
            if let Some(bus) = &self.active_bus_name {
                let _ = conn.call_method(
                    Some(bus.as_str()),
                    "/org/mpris/MediaPlayer2",
                    Some("org.mpris.MediaPlayer2.Player"),
                    "Previous",
                    &(),
                );
            }
            self.broadcast_method("Previous");
        }
    }

    fn broadcast_method(&self, method: &str) {
        let Some(conn) = &self.conn else { return };
        if let Ok(reply) = conn.call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "ListNames",
            &(),
        ) {
            if let Ok(names) = reply.body().deserialize::<Vec<String>>() {
                for n in names {
                    if n.starts_with("org.mpris.MediaPlayer2.") {
                        let _ = conn.call_method(
                            Some(n.as_str()),
                            "/org/mpris/MediaPlayer2",
                            Some("org.mpris.MediaPlayer2.Player"),
                            method,
                            &(),
                        );
                    }
                }
            }
        }
    }
}

fn extract_inner_str(v: &zbus::zvariant::Value) -> String {
    let s = format!("{v:?}");
    if let Some(pos) = s.find("Str(\"") {
        let sub = &s[pos + 5..];
        if let Some(end) = sub.find('"') {
            return sub[..end].to_string();
        }
    }
    if let Some(start) = s.find('"') {
        if let Some(end) = s[start + 1..].find('"') {
            return s[start + 1..start + 1 + end].to_string();
        }
    }
    String::new()
}

fn extract_inner_i64(v: &zbus::zvariant::Value) -> i64 {
    let s = format!("{v:?}");
    if let Some(start) = s.find("I64(") {
        let sub = &s[start + 4..];
        if let Some(end) = sub.find(')') {
            return sub[..end].trim().parse::<i64>().unwrap_or(0);
        }
    }
    0
}

fn extract_i64(val: &OwnedValue) -> i64 {
    let s = format!("{val:?}");
    if let Some(start) = s.find("I64(") {
        let sub = &s[start + 4..];
        if let Some(end) = sub.find(')') {
            return sub[..end].trim().parse::<i64>().unwrap_or(0);
        }
    }
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse::<i64>().unwrap_or(0)
}
