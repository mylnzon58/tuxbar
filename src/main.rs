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
use crate::types::{Color, Rect, WidgetAction};
use crate::widgets::clock::ClockWidget;
use crate::widgets::now_playing::NowPlayingWidget;
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
    println!("   Temp:    {:.1}°C", stats.temp_c);
    println!("   RAM:     {:.1} GB / {:.1} GB", stats.mem_used_mb as f32 / 1024.0, stats.mem_total_mb as f32 / 1024.0);
    println!("==================================================");
    Ok(())
}

fn run_test() -> Result<()> {
    println!("[TuxBar] Testing graphics and typography engine...");
    let font = FontEngine::new().map_err(|e| anyhow::anyhow!(e))?;
    let mut fb = Framebuffer::new();
    fb.clear(Color::BLACK);

    // Layout
    let media_rect = Rect::new(4, 2, 1200, 56);
    let stats_rect = Rect::new(1210, 2, 540, 56);
    let clock_rect = Rect::new(1756, 2, 248, 56);

    let mut now_playing = NowPlayingWidget::new();
    let mut stats_widget = SystemStatsWidget::new();
    let clock_widget = ClockWidget;

    let mut mpris = MprisClient::new();
    let track = mpris.update();
    now_playing.update_info(track.clone());

    let mut telemetry = TelemetryProvider::new();
    let stats = telemetry.refresh();
    stats_widget.update_stats(*stats);

    now_playing.render(&mut fb, media_rect, &font);
    stats_widget.render(&mut fb, stats_rect, &font);
    clock_widget.render(&mut fb, clock_rect, &font);

    println!("[TuxBar] Successfully rendered frame buffer (2008x60 ARGB8888, {} pixels)", fb.pixels.len());
    println!("[TuxBar] Graphics engine verification: PASSED ✓");
    Ok(())
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

    let mut mpris = MprisClient::new();
    let mut telemetry = TelemetryProvider::new();

    let mut now_playing = NowPlayingWidget::new();
    let mut stats_widget = SystemStatsWidget::new();
    let mut clock_widget = ClockWidget;

    // Layout regions
    let media_rect = Rect::new(4, 2, 1200, 56);
    let stats_rect = Rect::new(1210, 2, 540, 56);
    let clock_rect = Rect::new(1756, 2, 248, 56);

    let mut frame_count = 0u64;

    while running.load(Ordering::SeqCst) {
        // 1. Process touch events
        if let Some(ref mut listener) = touch {
            if let Ok(events) = listener.poll_events() {
                for event in events {
                    if media_rect.contains(event.x as i32, event.y as i32) {
                        match now_playing.handle_touch(&event, media_rect) {
                            WidgetAction::PlayPause => mpris.play_pause(),
                            WidgetAction::NextTrack => mpris.next(),
                            WidgetAction::PreviousTrack => mpris.previous(),
                            _ => {}
                        }
                    } else if clock_rect.contains(event.x as i32, event.y as i32) {
                        let _ = clock_widget.handle_touch(&event, clock_rect);
                    }
                }
            }
        }

        // 2. Refresh dynamic data
        if frame_count.is_multiple_of(15) {
            // Check MPRIS every 500ms
            let track = mpris.update();
            now_playing.update_info(track.clone());
        }

        if frame_count.is_multiple_of(30) {
            // Check CPU/Temp/RAM every 1000ms
            let stats = telemetry.refresh();
            stats_widget.update_stats(*stats);
        }

        now_playing.tick_scroll();

        // 3. Render frame
        fb.clear(Color::BLACK);
        now_playing.render(&mut fb, media_rect, &font);
        stats_widget.render(&mut fb, stats_rect, &font);
        clock_widget.render(&mut fb, clock_rect, &font);

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
