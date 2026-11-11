//! TuxBar — The Pure & Dynamic Touch Bar Engine for Linux
//! Author: Juan José Monzón (@mylnzon58)
//! License: GPL-3.0-or-later

mod core;
mod graphics;
mod services;
mod types;
mod widgets;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::core::{DrmBackend, KeyboardListener, TouchListener, VirtualKeyboard};
use crate::graphics::{FontEngine, Framebuffer};
use crate::services::{AudioCaptureService, MprisClient, TelemetryProvider};
use crate::types::{ActiveTouch, BarMode, Color, Rect, TouchPhase, WidgetAction};
use crate::widgets::clock::ClockWidget;
use crate::widgets::control_deck::ControlDeckWidget;
use crate::widgets::cyberdeck::CyberdeckWidget;
use crate::widgets::fn_keys::FnKeysWidget;
use crate::widgets::gallery::GalleryWidget;
use crate::widgets::now_playing::NowPlayingWidget;
use crate::widgets::spectrum::SpectrumVisualizerWidget;
use crate::widgets::system_stats::SystemStatsWidget;
use crate::widgets::Widget;

#[derive(Parser)]
#[command(name = "tuxbar")]
#[command(author = "Juan José Monzón (@mylnzon58)")]
#[command(version = "0.2.2")]
#[command(about = "The Pure & Dynamic Touch Bar Engine for Linux", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the TuxBar interactive daemon
    Run {
        /// DRM display device path
        #[arg(short, long, default_value = "/dev/dri/by-path/pci-0000:04:00.1-usb-0:6:2.1-card")]
        card: String,

        /// Touch digitizer device path
        #[arg(short, long, default_value = "/dev/input/by-path/pci-0000:04:00.1-usb-0:6:2.0-event")]
        input: String,
    },
    /// Render high-fidelity frame previews to PPM files
    Preview {
        /// Output prefix for preview files
        #[arg(short, long, default_value = "preview")]
        prefix: String,
    },
    /// Render a test buffer to verify graphics engine without capturing display
    Test,
    /// Display current media player and hardware telemetry in terminal
    Status,
    /// Send Play/Pause command to active media player
    PlayPause,
    /// Send Next Track command to active media player
    Next,
    /// Send Previous Track command to active media player
    Previous,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Run {
        card: "/dev/dri/by-path/pci-0000:04:00.1-usb-0:6:2.1-card".to_string(),
        input: "/dev/input/by-path/pci-0000:04:00.1-usb-0:6:2.0-event".to_string(),
    }) {
        Commands::Status => run_status(),
        Commands::PlayPause => {
            let mpris = MprisClient::new();
            mpris.play_pause();
            println!("[TuxBar] Play/Pause command dispatched via D-Bus");
            Ok(())
        }
        Commands::Next => {
            let mpris = MprisClient::new();
            mpris.next();
            println!("[TuxBar] Next track command dispatched via D-Bus");
            Ok(())
        }
        Commands::Previous => {
            let mpris = MprisClient::new();
            mpris.previous();
            println!("[TuxBar] Previous track command dispatched via D-Bus");
            Ok(())
        }
        Commands::Test => run_test(),
        Commands::Preview { prefix } => run_preview(&prefix),
        Commands::Run { card, input } => run_daemon(&card, &input),
    }
}

fn run_status() -> Result<()> {
    println!("==================================================");
    println!(" 🚀 TuxBar — Real-Time Subsystem Status");
    println!("==================================================");

    let mut mpris = MprisClient::new();
    let track = mpris.update();
    println!(" [🎵 Media]");
    println!("   Player:  {}", if track.player_name.is_empty() { "No active player detected" } else { &track.player_name });
    println!("   Status:  {}", if track.is_playing { "▶ Playing" } else { "❚❚ Paused / Idle" });
    println!("   Track:   {}", track.title);
    if !track.artist.is_empty() {
        println!("   Artist:  {}", track.artist);
    }
    if track.length_secs > 0.0 {
        println!("   Time:    {:.0}s / {:.0}s ({:.1}%)", track.position_secs, track.length_secs, (track.position_secs / track.length_secs) * 100.0);
    }

    let mut telemetry = TelemetryProvider::new();
    let stats = telemetry.refresh();
    println!("\n [⚡ Telemetry]");
    println!("   CPU:     {:.1}%", stats.cpu_usage_pct);
    println!("   Cores:   {} active threads", stats.core_count);
    for i in 0..stats.core_count.min(16) {
        println!("     Core {:>2}: {:>3.0}%", i, stats.core_usage_pct[i]);
    }
    println!("   Temp:    {:.1}°C", stats.temp_c);
    println!("   RAM:     {:.1} GB / {:.1} GB", stats.mem_used_mb as f32 / 1024.0, stats.mem_total_mb as f32 / 1024.0);

    let audio_capture = AudioCaptureService::new();
    thread::sleep(Duration::from_millis(150));
    let audio = audio_capture.get_telemetry();
    println!("\n [🔊 Real Audio Output Telemetry]");
    println!("   Status:  {}", if audio.is_silent { "Silent / No Audio Playing" } else { "🔊 Sound Active" });
    if !audio.is_silent {
        println!("   Decibels: {:.1} dBFS (RMS: {:.3})", audio.db, audio.rms);
    }
    println!("==================================================");
    Ok(())
}

fn run_preview(prefix: &str) -> Result<()> {
    println!("[TuxBar] Generating high-resolution previews...");
    let font = FontEngine::new().map_err(|e| anyhow::anyhow!(e))?;
    let mut fb = Framebuffer::new();

    let mut telemetry = TelemetryProvider::new();
    let stats = telemetry.refresh();

    let track = crate::services::mpris::TrackInfo {
        title: "Get Lucky".to_string(),
        artist: "Daft Punk".to_string(),
        player_name: "Spotify".to_string(),
        is_playing: true,
        length_secs: 238.0,
        position_secs: 84.0,
    };

    let mut now_playing = NowPlayingWidget::new();
    now_playing.update_info(track);

    let mut spectrum = SpectrumVisualizerWidget::new();
    spectrum.set_active(true);
    for _ in 0..15 {
        spectrum.update_frame();
    }

    let mut cyberdeck = CyberdeckWidget::new();
    cyberdeck.update_stats(*stats);

    let fn_keys = FnKeysWidget::new();
    let clock_widget = ClockWidget;
    let mode_btn_rect = Rect::new(1620, 1, 182, 58);
    let clock_rect = Rect::new(1806, 1, 198, 58);

    // 1. Preview Audiophile
    fb.clear(Color::BLACK);
    let media_rect = Rect::new(2, 1, 778, 58);
    let spec_rect = Rect::new(784, 1, 830, 58);
    now_playing.render(&mut fb, media_rect, &font);
    spectrum.render(&mut fb, spec_rect, &font);
    render_mode_button(&mut fb, mode_btn_rect, BarMode::Audiophile, &font);
    clock_widget.render(&mut fb, clock_rect, &font);
    fb.save_ppm(format!("{prefix}_audiophile.ppm"))?;

    // 2. Preview Cyberdeck
    fb.clear(Color::BLACK);
    let cyber_rect = Rect::new(2, 1, 1612, 58);
    cyberdeck.render(&mut fb, cyber_rect, &font);
    render_mode_button(&mut fb, mode_btn_rect, BarMode::Cyberdeck, &font);
    clock_widget.render(&mut fb, clock_rect, &font);
    fb.save_ppm(format!("{prefix}_cyberdeck.ppm"))?;

    // 3. Preview Fn Keys
    fb.clear(Color::BLACK);
    let fn_rect = Rect::new(2, 1, 1778, 58);
    let fn_mode_rect = Rect::new(1786, 1, 220, 58);
    fn_keys.render(&mut fb, fn_rect, &font);
    render_mode_button(&mut fb, fn_mode_rect, BarMode::FnKeys, &font);
    fb.save_ppm(format!("{prefix}_fnkeys.ppm"))?;

    // 4. Preview Pro Widgets
    fb.clear(Color::BLACK);
    let pro_media = Rect::new(2, 1, 920, 58);
    let pro_spec = Rect::new(926, 1, 350, 58);
    let mut stats_widget = SystemStatsWidget::new();
    stats_widget.update_stats(*stats);
    let pro_stats = Rect::new(1282, 1, 332, 58);

    now_playing.render(&mut fb, pro_media, &font);
    spectrum.render(&mut fb, pro_spec, &font);
    stats_widget.render(&mut fb, pro_stats, &font);
    render_mode_button(&mut fb, mode_btn_rect, BarMode::ProSplit, &font);
    clock_widget.render(&mut fb, clock_rect, &font);
    fb.save_ppm(format!("{prefix}_prosplit.ppm"))?;

    // 5. Preview Gallery
    fb.clear(Color::BLACK);
    let gallery = GalleryWidget::new();
    let gallery_rect = Rect::new(2, 1, 1612, 58);
    gallery.render(&mut fb, gallery_rect, &font);
    render_mode_button(&mut fb, mode_btn_rect, BarMode::Gallery, &font);
    clock_widget.render(&mut fb, clock_rect, &font);
    fb.save_ppm(format!("{prefix}_gallery.ppm"))?;

    // 6. Preview Control Deck
    fb.clear(Color::BLACK);
    let control_deck = ControlDeckWidget::new();
    let deck_rect = Rect::new(2, 1, 1778, 58);
    let deck_mode_rect = Rect::new(1786, 1, 220, 58);
    control_deck.render(&mut fb, deck_rect, &font);
    render_mode_button(&mut fb, deck_mode_rect, BarMode::ControlDeck, &font);
    fb.save_ppm(format!("{prefix}_controldeck.ppm"))?;

    println!("[TuxBar] Frame preview export complete ✓");
    Ok(())
}

fn run_test() -> Result<()> {
    run_preview("test")
}

fn dispatch_widget_action(
    action: &WidgetAction,
    virtual_kbd: &mut Option<VirtualKeyboard>,
    mpris: &MprisClient,
) {
    match action {
        WidgetAction::Escape => {
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_ESC);
            }
        }
        WidgetAction::BrightnessDown => {
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_BRIGHTNESSDOWN);
            }
            let _ = std::process::Command::new("brightnessctl")
                .args(["set", "5%-"])
                .spawn();
        }
        WidgetAction::BrightnessUp => {
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_BRIGHTNESSUP);
            }
            let _ = std::process::Command::new("brightnessctl")
                .args(["set", "+5%"])
                .spawn();
        }
        WidgetAction::VolumeDown => {
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_VOLUMEDOWN);
            }
            let _ = std::process::Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"])
                .spawn();
        }
        WidgetAction::VolumeUp => {
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_VOLUMEUP);
            }
            let _ = std::process::Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SINK@", "5%+"])
                .spawn();
        }
        WidgetAction::ToggleMute => {
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_MUTE);
            }
            let _ = std::process::Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                .spawn();
        }
        WidgetAction::ToggleMicMute => {
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_MICMUTE);
            }
            let _ = std::process::Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
                .spawn();
        }
        WidgetAction::LaunchCalc => {
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_CALC);
            }
            let _ = std::process::Command::new("systemd-run")
                .args(["--user", "--no-block", "gnome-calculator"])
                .spawn();
        }
        WidgetAction::LaunchBrowser => {
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_WWW);
            }
            let _ = std::process::Command::new("systemd-run")
                .args(["--user", "--no-block", "xdg-open", "https://google.com"])
                .spawn();
        }
        WidgetAction::LaunchFiles => {
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_FILE);
            }
            let _ = std::process::Command::new("systemd-run")
                .args(["--user", "--no-block", "nautilus"])
                .spawn();
        }
        WidgetAction::ToggleOverview => {
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_LEFTMETA);
            }
            let _ = std::process::Command::new("systemd-run")
                .args([
                    "--user",
                    "--no-block",
                    "gdbus",
                    "call",
                    "--session",
                    "--dest",
                    "org.gnome.Shell",
                    "--object-path",
                    "/org/gnome/Shell",
                    "--method",
                    "org.gnome.Shell.Eval",
                    "Main.overview.toggle()",
                ])
                .spawn();
        }
        WidgetAction::PlayPause => {
            mpris.play_pause();
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_PLAYPAUSE);
            }
        }
        WidgetAction::NextTrack => {
            mpris.next();
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_NEXTSONG);
            }
        }
        WidgetAction::PreviousTrack => {
            mpris.previous();
            if let Some(ref mut vk) = virtual_kbd {
                let _ = vk.click_key(evdev::Key::KEY_PREVIOUSSONG);
            }
        }
        _ => {}
    }
}

fn render_mode_button(fb: &mut Framebuffer, rect: Rect, mode: BarMode, font: &FontEngine) {
    fb.draw_rounded_rect(rect, 6, Color::rgb(45, 55, 72));
    let label = mode.label();
    let text_w = font.measure_text_width(label, 19.0);
    let text_x = rect.x + ((rect.width as i32 - text_w as i32) / 2);
    fb.draw_text(font, label, text_x, rect.y + 38, 19.0, Color::CYAN);
}

fn run_daemon(card_path: &str, input_path: &str) -> Result<()> {
    println!("=============================================================");
    println!(" 🚀 Starting TuxBar Dynamic Engine for Apple Touch Bar");
    println!(" Display: {}", card_path);
    println!(" Touch:   {}", input_path);
    println!("=============================================================");

    let font = FontEngine::new().map_err(|e| anyhow::anyhow!(e))?;
    let mut fb = Framebuffer::new();

    // Ensure Touch Bar backlight is powered on
    if let Ok(mut f) = std::fs::OpenOptions::new().write(true).open("/sys/class/backlight/appletb_backlight/brightness") {
        use std::io::Write;
        let _ = writeln!(f, "2");
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().write(true).open("/sys/class/backlight/appletb_backlight/bl_power") {
        use std::io::Write;
        let _ = writeln!(f, "0");
    }

    // Open DRM backend
    let mut drm = DrmBackend::open_touchbar(Path::new(card_path))?;
    println!("[TuxBar] DRM display hardware attached successfully.");

    // Open Touch listener (prefer auto-detection of hardware digitizer)
    let mut touch = TouchListener::find_touchbar_device()
        .or_else(|| TouchListener::open(Path::new(input_path)).ok());
    if touch.is_some() {
        println!("[TuxBar] Touchscreen digitizer attached successfully.");
    } else {
        println!("[TuxBar] Warning: Touchscreen in use or permissions missing; running in display-only mode.");
    }

    // Open Apple Internal Keyboard listener to detect physical Fn key
    let mut keyboard = KeyboardListener::find_apple_keyboard();
    if keyboard.is_some() {
        println!("[TuxBar] Apple keyboard listener attached (Fn key auto-switch enabled).");
    }

    // Open Virtual Keyboard for emitting real Esc, F1..F12 keystrokes
    let mut virtual_kbd = match VirtualKeyboard::new() {
        Ok(vk) => {
            println!("[TuxBar] Virtual Keyboard subsystem created via /dev/uinput.");
            Some(vk)
        }
        Err(e) => {
            println!("[TuxBar] Notice: /dev/uinput not available ({}), running virtual keys in visual-only mode.", e);
            None
        }
    };

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc_handler(r);

    let mut current_mode = BarMode::ProSplit;
    let mut previous_mode = BarMode::ProSplit;
    let mut fn_key_is_held = false;

    let audio_capture = AudioCaptureService::new();
    let mut mpris = MprisClient::new();
    let mut telemetry = TelemetryProvider::new();

    let mut now_playing = NowPlayingWidget::new();
    let mut spectrum = SpectrumVisualizerWidget::new();
    let mut cyberdeck = CyberdeckWidget::new();
    let mut gallery = GalleryWidget::new();
    let mut fn_keys = FnKeysWidget::new();
    let mut control_deck = ControlDeckWidget::new();
    let mut stats_widget = SystemStatsWidget::new();
    let mut clock_widget = ClockWidget;

    let mut active_touches: Vec<ActiveTouch> = Vec::new();
    let mut frame_count = 0u64;

    while running.load(Ordering::SeqCst) {
        // 1. Intercept physical keyboard Fn key
        if let Some(ref mut kbd) = keyboard {
            if let Some(held) = kbd.poll_fn_key() {
                if held && !fn_key_is_held {
                    fn_key_is_held = true;
                    if current_mode != BarMode::FnKeys {
                        previous_mode = current_mode;
                        current_mode = BarMode::FnKeys;
                        println!("[TuxBar] Physical Fn key pressed -> showing F1-F12 Keys");
                    }
                } else if !held && fn_key_is_held {
                    fn_key_is_held = false;
                    if current_mode == BarMode::FnKeys {
                        current_mode = previous_mode;
                        println!("[TuxBar] Physical Fn key released -> returning to {}", current_mode.label());
                    }
                }
            }
        }

        // Dynamic layout definitions based on mode (full 58px height)
        let (mode_btn_rect, clock_rect) = if current_mode == BarMode::FnKeys || current_mode == BarMode::ControlDeck {
            (Rect::new(1786, 1, 220, 58), Rect::new(0, 0, 0, 0))
        } else {
            (Rect::new(1620, 1, 182, 58), Rect::new(1806, 1, 198, 58))
        };

        // 2. Process touch events
        if let Some(ref mut listener) = touch {
            if let Ok(events) = listener.poll_events() {
                for event in events {
                    let px = event.x as i32;
                    let py = event.y as i32;

                    println!("[TuxBar] Touch: phase={:?}, pos=({}, {})", event.phase, px, py);

                    // Add touch glow effect point
                    if event.phase == TouchPhase::Down || event.phase == TouchPhase::Motion {
                        active_touches.retain(|t| (t.x - px).abs() > 30 || (t.y - py).abs() > 30);
                        active_touches.push(ActiveTouch {
                            x: px,
                            y: py,
                            intensity: 1.0,
                        });
                    }

                    // Check Mode Switcher button
                    if mode_btn_rect.contains(px, py) && event.phase == TouchPhase::Down {
                        current_mode = current_mode.next();
                        previous_mode = current_mode;
                        if current_mode == BarMode::Gallery {
                            gallery.load_from_pictures_or_fallback();
                        }
                        println!("[TuxBar] Switched to mode: {}", current_mode.label());
                        continue;
                    }

                    // Dispatch to mode-specific widgets
                    match current_mode {
                        BarMode::ControlDeck => {
                            let deck_rect = Rect::new(2, 1, 1778, 58);
                            if deck_rect.contains(px, py) {
                                let action = control_deck.handle_touch(&event, deck_rect);
                                if event.phase == TouchPhase::Down && action != WidgetAction::None {
                                    println!("[TuxBar] Control Deck Action Triggered: {:?}", action);
                                    dispatch_widget_action(&action, &mut virtual_kbd, &mpris);
                                }
                            }
                        }
                        BarMode::FnKeys => {
                            let fn_rect = Rect::new(2, 1, 1778, 58);
                            if fn_rect.contains(px, py) {
                                let _ = fn_keys.handle_touch(&event, fn_rect);
                                if event.phase == TouchPhase::Down {
                                    let count = 13;
                                    let gap = 5;
                                    let total_gaps = (count - 1) * gap;
                                    let btn_w = (fn_rect.width as i32 - total_gaps) / count;
                                    let offset_x = px - fn_rect.x;
                                    let stride = btn_w + gap;
                                    let idx = (offset_x / stride) as usize;
                                    if let Some(key) = FnKeysWidget::get_key_for_index(idx) {
                                        if let Some(ref mut vk) = virtual_kbd {
                                            let _ = vk.click_key(key);
                                        }
                                        println!("[TuxBar] Virtual Key Triggered: {:?}", key);
                                    }
                                }
                            }
                        }
                        BarMode::Audiophile => {
                            let media_rect = Rect::new(2, 1, 778, 58);
                            let spec_rect = Rect::new(784, 1, 830, 58);
                            if media_rect.contains(px, py) {
                                match now_playing.handle_touch(&event, media_rect) {
                                    WidgetAction::Escape => {
                                        if let Some(ref mut vk) = virtual_kbd {
                                            let _ = vk.click_key(evdev::Key::KEY_ESC);
                                        }
                                    }
                                    WidgetAction::PlayPause => {
                                        println!("[TuxBar] Media Action: Play/Pause triggered");
                                        mpris.play_pause();
                                        if let Some(ref mut vk) = virtual_kbd {
                                            let _ = vk.click_key(evdev::Key::KEY_PLAYPAUSE);
                                        }
                                    }
                                    WidgetAction::NextTrack => {
                                        println!("[TuxBar] Media Action: Next Track triggered");
                                        mpris.next();
                                        if let Some(ref mut vk) = virtual_kbd {
                                            let _ = vk.click_key(evdev::Key::KEY_NEXTSONG);
                                        }
                                    }
                                    WidgetAction::PreviousTrack => {
                                        println!("[TuxBar] Media Action: Previous Track triggered");
                                        mpris.previous();
                                        if let Some(ref mut vk) = virtual_kbd {
                                            let _ = vk.click_key(evdev::Key::KEY_PREVIOUSSONG);
                                        }
                                    }
                                    _ => {}
                                }
                            } else if spec_rect.contains(px, py) {
                                let _ = spectrum.handle_touch(&event, spec_rect);
                            }
                        }
                        BarMode::Cyberdeck => {
                            let cyber_rect = Rect::new(2, 1, 1612, 58);
                            if cyber_rect.contains(px, py)
                                && cyberdeck.handle_touch(&event, cyber_rect) == WidgetAction::ToggleMode
                            {
                                current_mode = current_mode.next();
                                previous_mode = current_mode;
                            }
                        }
                        BarMode::ProSplit => {
                            let is_playing = now_playing.info.is_playing;
                            let pro_media = if is_playing {
                                Rect::new(2, 1, 920, 58)
                            } else {
                                Rect::new(2, 1, 1280, 58)
                            };
                            let pro_spec = Rect::new(926, 1, 350, 58);
                            if pro_media.contains(px, py) {
                                match now_playing.handle_touch(&event, pro_media) {
                                    WidgetAction::Escape => {
                                        if let Some(ref mut vk) = virtual_kbd {
                                            let _ = vk.click_key(evdev::Key::KEY_ESC);
                                        }
                                    }
                                    WidgetAction::PlayPause => {
                                        println!("[TuxBar] Media Action: Play/Pause triggered");
                                        mpris.play_pause();
                                        if let Some(ref mut vk) = virtual_kbd {
                                            let _ = vk.click_key(evdev::Key::KEY_PLAYPAUSE);
                                        }
                                    }
                                    WidgetAction::NextTrack => {
                                        println!("[TuxBar] Media Action: Next Track triggered");
                                        mpris.next();
                                        if let Some(ref mut vk) = virtual_kbd {
                                            let _ = vk.click_key(evdev::Key::KEY_NEXTSONG);
                                        }
                                    }
                                    WidgetAction::PreviousTrack => {
                                        println!("[TuxBar] Media Action: Previous Track triggered");
                                        mpris.previous();
                                        if let Some(ref mut vk) = virtual_kbd {
                                            let _ = vk.click_key(evdev::Key::KEY_PREVIOUSSONG);
                                        }
                                    }
                                    _ => {}
                                }
                            } else if is_playing && pro_spec.contains(px, py) && event.phase == TouchPhase::Down {
                                mpris.play_pause();
                            }
                        }
                        BarMode::Gallery => {
                            let gallery_rect = Rect::new(2, 1, 1612, 58);
                            if gallery_rect.contains(px, py) {
                                let _ = gallery.handle_touch(&event, gallery_rect);
                            }
                        }
                    }

                    if clock_rect.contains(px, py) && event.phase == TouchPhase::Down {
                        let _ = clock_widget.handle_touch(&event, clock_rect);
                        current_mode = current_mode.next();
                        previous_mode = current_mode;
                    }
                }
            }
        }

        // 3. Refresh dynamic data
        let audio_telem = audio_capture.get_telemetry();
        spectrum.update_with_real_audio(&audio_telem);

        if frame_count.is_multiple_of(15) {
            let track = mpris.update();
            now_playing.update_info(track.clone());
        }

        if frame_count.is_multiple_of(30) {
            let stats = telemetry.refresh();
            cyberdeck.update_stats(*stats);
            stats_widget.update_stats(*stats);
        }

        now_playing.tick_scroll();
        cyberdeck.tick();

        // 4. Render Layout onto Framebuffer
        fb.clear(Color::BLACK);

        match current_mode {
            BarMode::ControlDeck => {
                let deck_rect = Rect::new(2, 1, 1778, 58);
                control_deck.render(&mut fb, deck_rect, &font);
            }
            BarMode::FnKeys => {
                let fn_rect = Rect::new(2, 1, 1778, 58);
                fn_keys.render(&mut fb, fn_rect, &font);
            }
            BarMode::Audiophile => {
                let media_rect = Rect::new(2, 1, 778, 58);
                let spec_rect = Rect::new(784, 1, 830, 58);
                now_playing.render(&mut fb, media_rect, &font);
                spectrum.render(&mut fb, spec_rect, &font);
            }
            BarMode::Cyberdeck => {
                let cyber_rect = Rect::new(2, 1, 1612, 58);
                cyberdeck.render(&mut fb, cyber_rect, &font);
            }
            BarMode::ProSplit => {
                let is_playing = now_playing.info.is_playing;
                if is_playing {
                    let pro_media = Rect::new(2, 1, 920, 58);
                    let pro_spec = Rect::new(926, 1, 350, 58);
                    let pro_stats = Rect::new(1282, 1, 332, 58);

                    now_playing.render(&mut fb, pro_media, &font);
                    spectrum.render(&mut fb, pro_spec, &font);
                    stats_widget.render(&mut fb, pro_stats, &font);
                } else {
                    let pro_media = Rect::new(2, 1, 1280, 58);
                    let pro_stats = Rect::new(1286, 1, 328, 58);

                    now_playing.render(&mut fb, pro_media, &font);
                    stats_widget.render(&mut fb, pro_stats, &font);
                }
            }
            BarMode::Gallery => {
                let gallery_rect = Rect::new(2, 1, 1612, 58);
                gallery.render(&mut fb, gallery_rect, &font);
            }
        }

        // Draw mode button and clock
        render_mode_button(&mut fb, mode_btn_rect, current_mode, &font);
        if clock_rect.width > 0 {
            clock_widget.render(&mut fb, clock_rect, &font);
        }

        // Render luminous touch glow indicators
        for touch in &mut active_touches {
            let alpha = (touch.intensity * 200.0) as u8;
            fb.draw_glow_circle(touch.x, touch.y, 22, Color::rgba(0, 240, 255, alpha));
            touch.intensity -= 0.15;
        }
        active_touches.retain(|t| t.intensity > 0.05);

        // 5. Submit to DRM hardware
        let _ = drm.draw_pixels(&fb.pixels);

        frame_count += 1;
        thread::sleep(Duration::from_millis(33)); // ~30 FPS
    }

    println!("[TuxBar] Cleaning up display and exiting safely.");
    fb.clear(Color::BLACK);
    let _ = drm.draw_pixels(&fb.pixels);
    Ok(())
}

fn ctrlc_handler(running: Arc<AtomicBool>) {
    unsafe {
        let _ = nix::sys::signal::signal(
            nix::sys::signal::Signal::SIGINT,
            nix::sys::signal::SigHandler::Handler(handle_sig),
        );
        let _ = nix::sys::signal::signal(
            nix::sys::signal::Signal::SIGTERM,
            nix::sys::signal::SigHandler::Handler(handle_sig),
        );
    }
    GLOBAL_RUNNING.store(true, Ordering::SeqCst);
    RUNNING_REF.store(Arc::into_raw(running) as usize, Ordering::SeqCst);
}

static GLOBAL_RUNNING: AtomicBool = AtomicBool::new(false);
static RUNNING_REF: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

extern "C" fn handle_sig(_: libc::c_int) {
    let ptr = RUNNING_REF.load(Ordering::SeqCst);
    if ptr != 0 {
        let arc = unsafe { Arc::from_raw(ptr as *const AtomicBool) };
        arc.store(false, Ordering::SeqCst);
        let _ = Arc::into_raw(arc);
    }
}
