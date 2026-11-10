//! Touch Bar digitizer input listener using Linux evdev subsystem
//! Operates in strict non-blocking mode with full Multi-Touch Protocol Type-B support.

use crate::types::{TouchEvent, TouchPhase};
use anyhow::Result;
use evdev::{AbsoluteAxisType, Device, EventType, Key};
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct TouchListener {
    device: Device,
    max_raw_x: f32,
    max_raw_y: f32,
    
    // Position state
    cur_x: u32,
    cur_y: u32,
    last_reported_x: u32,
    last_reported_y: u32,
    
    // Contact state
    is_down: bool,
    pending_down: bool,
    pending_up: bool,
    pos_updated: bool,
}

impl TouchListener {
    pub fn open(path: &Path) -> Result<Self> {
        let device = Device::open(path)?;

        // Ensure non-blocking mode so it never stalls rendering
        unsafe {
            let fd = device.as_raw_fd();
            let flags = libc::fcntl(fd, libc::F_GETFL, 0);
            libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        // Apple Touch Bar hardware reported boundaries (0..32767 X, 0..127 Y)
        let max_x = 32767.0f32;
        let max_y = 127.0f32;

        Ok(Self {
            device,
            max_raw_x: max_x,
            max_raw_y: max_y,
            cur_x: 0,
            cur_y: 0,
            last_reported_x: 0,
            last_reported_y: 0,
            is_down: false,
            pending_down: false,
            pending_up: false,
            pos_updated: false,
        })
    }

    /// Automatically find the Touch Bar touchpad among /dev/input/event*
    pub fn find_touchbar_device() -> Option<Self> {
        for (path, dev) in evdev::enumerate() {
            let name = dev.name().unwrap_or("").to_lowercase();
            if name.contains("touch bar") && name.contains("touchpad") {
                // Ignore virtual proxies like linux-3-finger-drag
                if let Some(phys) = dev.physical_path() {
                    if phys.contains("proxy") || phys.contains("virtual") {
                        continue;
                    }
                }
                if let Ok(listener) = Self::open(&path) {
                    println!("[TuxBar] Found hardware Touch Bar digitizer at {:?}", path);
                    return Some(listener);
                }
            }
        }
        None
    }

    pub fn poll_events(&mut self) -> Result<Vec<TouchEvent>> {
        let mut events = Vec::new();

        match self.device.fetch_events() {
            Ok(iterator) => {
                for ev in iterator {
                    match ev.event_type() {
                        EventType::ABSOLUTE => {
                            let code = AbsoluteAxisType(ev.code());
                            if code == AbsoluteAxisType::ABS_MT_POSITION_X || code == AbsoluteAxisType::ABS_X {
                                let normalized = ((ev.value() as f32 / self.max_raw_x) * 2008.0).clamp(0.0, 2007.0);
                                self.cur_x = normalized as u32;
                                self.pos_updated = true;
                            } else if code == AbsoluteAxisType::ABS_MT_POSITION_Y || code == AbsoluteAxisType::ABS_Y {
                                let normalized = ((ev.value() as f32 / self.max_raw_y) * 60.0).clamp(0.0, 59.0);
                                self.cur_y = normalized as u32;
                                self.pos_updated = true;
                            } else if code == AbsoluteAxisType::ABS_MT_TRACKING_ID {
                                let tracking_id = ev.value();
                                if tracking_id >= 0 {
                                    self.pending_down = true;
                                    self.pending_up = false;
                                } else {
                                    self.pending_up = true;
                                    self.pending_down = false;
                                }
                            }
                        }
                        EventType::KEY => {
                            let key = Key::new(ev.code());
                            if key == Key::BTN_TOUCH {
                                if ev.value() == 1 {
                                    self.pending_down = true;
                                    self.pending_up = false;
                                } else {
                                    self.pending_up = true;
                                    self.pending_down = false;
                                }
                            }
                        }
                        EventType::SYNCHRONIZATION => {
                            // Frame complete: Dispatch touch event with current coordinates
                            if self.pending_down {
                                self.is_down = true;
                                self.pending_down = false;
                                self.last_reported_x = self.cur_x;
                                self.last_reported_y = self.cur_y;
                                events.push(TouchEvent {
                                    slot: 0,
                                    x: self.cur_x,
                                    y: self.cur_y,
                                    phase: TouchPhase::Down,
                                });
                            } else if self.pending_up {
                                self.is_down = false;
                                self.pending_up = false;
                                events.push(TouchEvent {
                                    slot: 0,
                                    x: self.cur_x,
                                    y: self.cur_y,
                                    phase: TouchPhase::Up,
                                });
                            } else if self.is_down && self.pos_updated 
                                && ((self.cur_x as i32 - self.last_reported_x as i32).abs() > 2 
                                    || (self.cur_y as i32 - self.last_reported_y as i32).abs() > 2) 
                            {
                                self.last_reported_x = self.cur_x;
                                self.last_reported_y = self.cur_y;
                                events.push(TouchEvent {
                                    slot: 0,
                                    x: self.cur_x,
                                    y: self.cur_y,
                                    phase: TouchPhase::Motion,
                                });
                            }
                            self.pos_updated = false;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e.into()),
        }

        Ok(events)
    }
}
