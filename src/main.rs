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

use crate::core::{DrmBackend, TouchListener};
use crate::graphics::{FontEngine, Framebuffer};
use crate::services::{MprisClient, TelemetryProvider};
use crate::types::{ActiveTouch, BarMode, Color, Rect, TouchPhase, WidgetAction};
use crate::widgets::clock::ClockWidget;
use crate::widgets::cyberdeck::CyberdeckWidget;
use crate::widgets::gallery::GalleryWidget;
use crate::widgets::now_playing::NowPlayingWidget;
use crate::widgets::spectrum::SpectrumVisualizerWidget;
use crate::widgets::system_stats::SystemStatsWidget;
use crate::widgets::Widget;

#[derive(Parser)]
#[command(name = "tuxbar")]
#[command(author = "Juan José Monzón (@mylnzon58)")]
#[command(version = "0.1.0")]
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
    println!("==================================================");
    Ok(())
}

fn run_preview(prefix: &str) -> Result<()> {
    println!("[TuxBar] Generating high-resolution preview frames for all 3 modes...");
    let font = FontEngine::new().map_err(|e| anyhow::anyhow!(e))?;
    let mut fb = Framebuffer::new();

    let mut telemetry = TelemetryProvider::new();
    let stats = telemetry.refresh();

    let mut mpris = MprisClient::new();
    let mut track = mpris.update().clone();
    if track.title.is_empty() {
        track.player_name = "Spotify".to_string();
        track.title = "Around the World (Remastered)".to_string();
        track.artist = "Daft Punk".to_string();
        track.is_playing = true;
        track.length_secs = 238.0;
        track.position_secs = 84.0;
    }

    let mut now_playing = NowPlayingWidget::new();
    now_playing.update_info(track);

    let mut spectrum = SpectrumVisualizerWidget::new();
    spectrum.set_active(true);
    for _ in 0..15 {
        spectrum.update_frame();
    }

    let mut cyberdeck = CyberdeckWidget::new();
    cyberdeck.update_stats(*stats);

    let clock_widget = ClockWidget;
    let mode_btn_rect = Rect::new(1610, 8, 136, 44);
    let clock_rect = Rect::new(1756, 2, 248, 56);

    // 1. Preview Audiophile
    fb.clear(Color::BLACK);
    let media_rect = Rect::new(4, 2, 720, 56);
    let spec_rect = Rect::new(730, 2, 870, 56);
    now_playing.render(&mut fb, media_rect, &font);
    spectrum.render(&mut fb, spec_rect, &font);
    render_mode_button(&mut fb, mode_btn_rect, BarMode::Audiophile, &font);
    clock_widget.render(&mut fb, clock_rect, &font);
    let path_audio = format!("{prefix}_audiophile.ppm");
    fb.save_ppm(&path_audio)?;
    println!("[TuxBar] Saved {}", path_audio);

    // 2. Preview Cyberdeck
    fb.clear(Color::BLACK);
    let cyber_rect = Rect::new(4, 2, 1596, 56);
    cyberdeck.render(&mut fb, cyber_rect, &font);
    render_mode_button(&mut fb, mode_btn_rect, BarMode::Cyberdeck, &font);
    clock_widget.render(&mut fb, clock_rect, &font);
    let path_cyber = format!("{prefix}_cyberdeck.ppm");
    fb.save_ppm(&path_cyber)?;
    println!("[TuxBar] Saved {}", path_cyber);

    // 3. Preview Pro Split
    fb.clear(Color::BLACK);
    let pro_media = Rect::new(4, 2, 840, 56);
    let pro_spec = Rect::new(850, 2, 380, 56);
    let mut stats_widget = SystemStatsWidget::new();
    stats_widget.update_stats(*stats);
    let pro_stats = Rect::new(1240, 2, 360, 56);
    now_playing.render(&mut fb, pro_media, &font);
    spectrum.render(&mut fb, pro_spec, &font);
    stats_widget.render(&mut fb, pro_stats, &font);
    render_mode_button(&mut fb, mode_btn_rect, BarMode::ProSplit, &font);
    clock_widget.render(&mut fb, clock_rect, &font);
    let path_pro = format!("{prefix}_prosplit.ppm");
    fb.save_ppm(&path_pro)?;
    println!("[TuxBar] Saved {}", path_pro);

    // 4. Preview Gallery Scrubber
    fb.clear(Color::BLACK);
    let gallery_rect = Rect::new(4, 2, 1596, 56);
    let gallery = GalleryWidget::new();
    gallery.render(&mut fb, gallery_rect, &font);
    render_mode_button(&mut fb, mode_btn_rect, BarMode::Gallery, &font);
    clock_widget.render(&mut fb, clock_rect, &font);
    let path_gallery = format!("{prefix}_gallery.ppm");
    fb.save_ppm(&path_gallery)?;
    println!("[TuxBar] Saved {}", path_gallery);

    println!("[TuxBar] Frame preview export complete ✓");
    Ok(())
}

fn run_test() -> Result<()> {
    run_preview("test")
}

fn render_mode_button(fb: &mut Framebuffer, rect: Rect, mode: BarMode, font: &FontEngine) {
    fb.draw_rounded_rect(rect, 6, Color::DARK_GRAY);
    let label = mode.label();
    let text_w = font.measure_text_width(label, 11.0);
    let text_x = rect.x + ((rect.width as i32 - text_w as i32) / 2);
    fb.draw_text(font, label, text_x, rect.y + 27, 11.0, Color::CYAN);
}

fn run_daemon(card_path: &str, input_path: &str) -> Result<()> {
    println!("=============================================================");
    println!(" 🚀 Starting TuxBar Dynamic Engine for Apple Touch Bar");
    println!(" Display: {}", card_path);
    println!(" Touch:   {}", input_path);
    println!("=============================================================");

    let font = FontEngine::new().map_err(|e| anyhow::anyhow!(e))?;
    let mut fb = Framebuffer::new();

    // Open DRM backend
    let mut drm = DrmBackend::open_touchbar(Path::new(card_path))?;
    println!("[TuxBar] DRM display hardware attached successfully.");

    // Open Touch listener (non-fatal if input device is busy)
    let mut touch = TouchListener::open(Path::new(input_path)).ok();
    if touch.is_some() {
        println!("[TuxBar] Touchscreen digitizer attached successfully.");
    } else {
        println!("[TuxBar] Warning: Touchscreen in use or permissions missing; running in display-only mode.");
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc_handler(r);

    let mut current_mode = BarMode::ProSplit;
    let mut mpris = MprisClient::new();
    let mut telemetry = TelemetryProvider::new();

    let mut now_playing = NowPlayingWidget::new();
    let mut spectrum = SpectrumVisualizerWidget::new();
    let mut cyberdeck = CyberdeckWidget::new();
    let mut gallery = GalleryWidget::new();
    let mut stats_widget = SystemStatsWidget::new();
    let mut clock_widget = ClockWidget;

    // Common layout regions
    let mode_btn_rect = Rect::new(1610, 8, 136, 44);
    let clock_rect = Rect::new(1756, 2, 248, 56);

    let mut active_touches: Vec<ActiveTouch> = Vec::new();
    let mut frame_count = 0u64;

    while running.load(Ordering::SeqCst) {
        // 1. Process touch events
        if let Some(ref mut listener) = touch {
            if let Ok(events) = listener.poll_events() {
                for event in events {
                    let px = event.x as i32;
                    let py = event.y as i32;

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
                        println!("[TuxBar] Switched to mode: {}", current_mode.label());
                        continue;
                    }

                    // Dispatch to mode-specific widgets
                    match current_mode {
                        BarMode::Audiophile => {
                            let media_rect = Rect::new(4, 2, 720, 56);
                            let spec_rect = Rect::new(730, 2, 870, 56);
                            if media_rect.contains(px, py) {
                                match now_playing.handle_touch(&event, media_rect) {
                                    WidgetAction::PlayPause => mpris.play_pause(),
                                    WidgetAction::NextTrack => mpris.next(),
                                    WidgetAction::PreviousTrack => mpris.previous(),
                                    _ => {}
                                }
                            } else if spec_rect.contains(px, py) {
                                let _ = spectrum.handle_touch(&event, spec_rect);
                            }
                        }
                        BarMode::Cyberdeck => {
                            let cyber_rect = Rect::new(4, 2, 1596, 56);
                            if cyber_rect.contains(px, py) {
                                let _ = cyberdeck.handle_touch(&event, cyber_rect);
                            }
                        }
                        BarMode::ProSplit => {
                            let pro_media = Rect::new(4, 2, 840, 56);
                            if pro_media.contains(px, py) {
                                match now_playing.handle_touch(&event, pro_media) {
                                    WidgetAction::PlayPause => mpris.play_pause(),
                                    WidgetAction::NextTrack => mpris.next(),
                                    WidgetAction::PreviousTrack => mpris.previous(),
                                    _ => {}
                                }
                            }
                        }
                        BarMode::Gallery => {
                            let gallery_rect = Rect::new(4, 2, 1596, 56);
                            if gallery_rect.contains(px, py) {
                                let _ = gallery.handle_touch(&event, gallery_rect);
                            }
                        }
                    }

                    if clock_rect.contains(px, py) {
                        let _ = clock_widget.handle_touch(&event, clock_rect);
                    }
                }
            }
        }

        // 2. Refresh dynamic data
        if frame_count.is_multiple_of(15) {
            let track = mpris.update();
            spectrum.set_active(track.is_playing);
            now_playing.update_info(track.clone());
        }

        if frame_count.is_multiple_of(30) {
            let stats = telemetry.refresh();
            cyberdeck.update_stats(*stats);
            stats_widget.update_stats(*stats);
        }

        spectrum.update_frame();
        cyberdeck.tick();
        now_playing.tick_scroll();

        // 3. Render frame according to active mode
        fb.clear(Color::BLACK);

        match current_mode {
            BarMode::Audiophile => {
                let media_rect = Rect::new(4, 2, 720, 56);
                let spec_rect = Rect::new(730, 2, 870, 56);
                now_playing.render(&mut fb, media_rect, &font);
                spectrum.render(&mut fb, spec_rect, &font);
            }
            BarMode::Cyberdeck => {
                let cyber_rect = Rect::new(4, 2, 1596, 56);
                cyberdeck.render(&mut fb, cyber_rect, &font);
            }
            BarMode::ProSplit => {
                let pro_media = Rect::new(4, 2, 840, 56);
                let pro_spec = Rect::new(850, 2, 380, 56);
                let pro_stats = Rect::new(1240, 2, 360, 56);
                now_playing.render(&mut fb, pro_media, &font);
                spectrum.render(&mut fb, pro_spec, &font);
                stats_widget.render(&mut fb, pro_stats, &font);
            }
            BarMode::Gallery => {
                let gallery_rect = Rect::new(4, 2, 1596, 56);
                gallery.render(&mut fb, gallery_rect, &font);
            }
        }

        // Draw mode button and clock
        render_mode_button(&mut fb, mode_btn_rect, current_mode, &font);
        clock_widget.render(&mut fb, clock_rect, &font);

        // Render luminous touch glow indicators
        for touch in &mut active_touches {
            let alpha = (touch.intensity * 180.0) as u8;
            fb.draw_glow_circle(touch.x, touch.y, 20, Color::rgba(0, 240, 255, alpha));
            touch.intensity -= 0.15;
        }
        active_touches.retain(|t| t.intensity > 0.05);

        // 4. Submit to DRM hardware
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
