//! Real-Time Live Audio Capture and Frequency Analyzer for Linux (PipeWire / PulseAudio)
//! Captures actual sound playing through the speakers and calculates real RMS, dB, and 32 frequency bands.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct AudioTelemetry {
    pub is_silent: bool,
    pub db: f32,
    pub rms: f32,
    pub bands: [f32; 32],
}

impl Default for AudioTelemetry {
    fn default() -> Self {
        Self {
            is_silent: true,
            db: -100.0,
            rms: 0.0,
            bands: [0.0; 32],
        }
    }
}

pub struct AudioCaptureService {
    telemetry: Arc<Mutex<AudioTelemetry>>,
    #[allow(dead_code)]
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl AudioCaptureService {
    #[must_use]
    pub fn new() -> Self {
        let telemetry = Arc::new(Mutex::new(AudioTelemetry::default()));
        let telem_clone = telemetry.clone();

        let worker_handle = thread::spawn(move || {
            capture_loop(telem_clone);
        });

        Self {
            telemetry,
            worker_handle: Some(worker_handle),
        }
    }

    pub fn get_telemetry(&self) -> AudioTelemetry {
        self.telemetry.lock().map(|t| *t).unwrap_or_default()
    }
}

fn capture_loop(telemetry: Arc<Mutex<AudioTelemetry>>) {
    // 32 target frequencies exponentially distributed from 60 Hz (deep bass) to 3800 Hz (treble)
    let mut freqs = [0.0f32; 32];
    for i in 0..32 {
        let t = i as f32 / 31.0;
        freqs[i] = 60.0 * (3800.0 / 60.0f32).powf(t);
    }

    loop {
        let child_res = spawn_audio_process();
        match child_res {
            Ok(mut child) => {
                if let Some(mut stdout) = child.stdout.take() {
                    let chunk_size = 256; // 32ms at 8000Hz (31.25 FPS)
                    let mut byte_buf = vec![0u8; chunk_size * 2];

                    while stdout.read_exact(&mut byte_buf).is_ok() {
                        let mut samples = [0.0f32; 256];
                        let mut sum_sq = 0.0f32;

                        for (i, chunk) in byte_buf.chunks_exact(2).enumerate() {
                            let s = i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0;
                            samples[i] = s;
                            sum_sq += s * s;
                        }

                        let rms = (sum_sq / chunk_size as f32).sqrt();
                        let is_silent = rms < 0.003;
                        let db = if is_silent {
                            -100.0
                        } else {
                            (20.0 * (rms + 1e-6).log10()).clamp(-80.0, 0.0)
                        };

                        let mut band_values = [0.0f32; 32];
                        if !is_silent {
                            // Compute magnitude at each frequency band using correlation
                            for (b, &freq) in freqs.iter().enumerate() {
                                let w = 2.0 * std::f32::consts::PI * freq / 8000.0;
                                let mut re = 0.0f32;
                                let mut im = 0.0f32;
                                for (n, &s) in samples.iter().enumerate() {
                                    let phase = w * n as f32;
                                    re += s * phase.cos();
                                    im -= s * phase.sin();
                                }
                                let mag = (re * re + im * im).sqrt() / (chunk_size as f32 / 4.0);
                                let weight = if b < 8 { 1.3 } else if b < 20 { 1.0 } else { 0.8 };
                                band_values[b] = (mag * weight).clamp(0.0, 1.0);
                            }
                        }

                        if let Ok(mut t) = telemetry.lock() {
                            t.is_silent = is_silent;
                            t.rms = rms;
                            t.db = db;
                            t.bands = band_values;
                        }
                    }
                }
                let _ = child.kill();
            }
            Err(_) => {
                // If audio service is not running or suspended, wait and retry
                thread::sleep(Duration::from_secs(2));
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}

fn spawn_audio_process() -> std::io::Result<Child> {
    let uid = unsafe { libc::geteuid() };
    if uid == 0 {
        let user = std::env::var("SUDO_USER").unwrap_or_else(|_| "zeeroj".to_string());
        if let Ok(child) = Command::new("su")
            .args([
                "-",
                &user,
                "-c",
                "pw-record --target @DEFAULT_AUDIO_SINK@ - --rate=8000 --channels=1 --format=s16 2>/dev/null || pw-record --target effect_input.filter-chain-speakers - --rate=8000 --channels=1 --format=s16",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            return Ok(child);
        }
    }

    // Direct user execution
    Command::new("pw-record")
        .args([
            "--target",
            "@DEFAULT_AUDIO_SINK@",
            "-",
            "--rate=8000",
            "--channels=1",
            "--format=s16",
        ])
        .env("XDG_RUNTIME_DIR", "/run/user/1000")
        .env("PIPEWIRE_RUNTIME_DIR", "/run/user/1000")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
}
