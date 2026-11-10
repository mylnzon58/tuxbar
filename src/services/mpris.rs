//! MPRIS D-Bus client for discovering and controlling media players
//! Works with Spotify, Chrome, Firefox, VLC, and any MPRIS compliant player.

use std::collections::HashMap;
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

        // 1. Discover active players if needed
        if self.active_bus_name.is_none() {
            if let Ok(reply) = conn.call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "ListNames",
                &(),
            ) {
                if let Ok(names) = reply.body().deserialize::<Vec<String>>() {
                    for name in names {
                        if name.starts_with("org.mpris.MediaPlayer2.") {
                            self.active_bus_name = Some(name);
                            break;
                        }
                    }
                }
            }
        }

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
            if let Ok(val) = meta_reply.body().deserialize::<HashMap<String, OwnedValue>>() {
                if let Some(t) = val.get("xesam:title") {
                    title = extract_str(t);
                }
                if let Some(a) = val.get("xesam:artist") {
                    artist = extract_str(a);
                }
                if let Some(len) = val.get("mpris:length") {
                    let l = extract_i64(len);
                    if l > 0 {
                        length_secs = (l as f32) / 1_000_000.0;
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
        if let (Some(conn), Some(bus)) = (&self.conn, &self.active_bus_name) {
            let _ = conn.call_method(
                Some(bus.as_str()),
                "/org/mpris/MediaPlayer2",
                Some("org.mpris.MediaPlayer2.Player"),
                "PlayPause",
                &(),
            );
        }
    }

    pub fn next(&self) {
        if let (Some(conn), Some(bus)) = (&self.conn, &self.active_bus_name) {
            let _ = conn.call_method(
                Some(bus.as_str()),
                "/org/mpris/MediaPlayer2",
                Some("org.mpris.MediaPlayer2.Player"),
                "Next",
                &(),
            );
        }
    }

    pub fn previous(&self) {
        if let (Some(conn), Some(bus)) = (&self.conn, &self.active_bus_name) {
            let _ = conn.call_method(
                Some(bus.as_str()),
                "/org/mpris/MediaPlayer2",
                Some("org.mpris.MediaPlayer2.Player"),
                "Previous",
                &(),
            );
        }
    }
}

fn extract_str(val: &OwnedValue) -> String {
    let raw = format!("{val:?}");
    if let Some(start) = raw.find('"') {
        if let Some(end) = raw[start + 1..].find('"') {
            return raw[start + 1..start + 1 + end].to_string();
        }
    }
    raw
}

fn extract_i64(val: &OwnedValue) -> i64 {
    let raw = format!("{val:?}");
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse::<i64>().unwrap_or(0)
}
