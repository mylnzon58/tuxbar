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
    current_slot: u32,

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

        // Query reported boundaries directly from hardware axis metadata
        let (max_x, max_y, init_x, init_y) = if let Ok(state) = device.get_abs_state() {
            let abs_x = &state[AbsoluteAxisType::ABS_X.0 as usize];
            let abs_y = &state[AbsoluteAxisType::ABS_Y.0 as usize];
            let mx = if abs_x.maximum > 0 { abs_x.maximum as f32 } else { 32767.0 };
            let my = if abs_y.maximum > 0 { abs_y.maximum as f32 } else { 127.0 };
            let ix = ((abs_x.value as f32 / mx) * 2008.0).clamp(0.0, 2007.0) as u32;
            let iy = ((abs_y.value as f32 / my) * 60.0).clamp(0.0, 59.0) as u32;
            (mx, my, ix, iy)
        } else {
            (32767.0, 127.0, 0, 0)
        };

        println!("[TuxBar] Touch digitizer initialized (axes: max_x={:.0}, max_y={:.0})", max_x, max_y);

        Ok(Self {
            device,
            max_raw_x: max_x,
            max_raw_y: max_y,
            current_slot: 0,
            cur_x: init_x,
            cur_y: init_y,
            last_reported_x: init_x,
            last_reported_y: init_y,
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
                            match code {
                                AbsoluteAxisType::ABS_MT_SLOT => {
                                    self.current_slot = ev.value() as u32;
                                }
                                AbsoluteAxisType::ABS_X => {
                                    let normalized = ((ev.value() as f32 / self.max_raw_x) * 2008.0).clamp(0.0, 2007.0);
                                    self.cur_x = normalized as u32;
                                    self.pos_updated = true;
                                }
                                AbsoluteAxisType::ABS_Y => {
                                    let normalized = ((ev.value() as f32 / self.max_raw_y) * 60.0).clamp(0.0, 59.0);
                                    self.cur_y = normalized as u32;
                                    self.pos_updated = true;
                                }
                                AbsoluteAxisType::ABS_MT_POSITION_X => {
                                    if self.current_slot == 0 {
                                        let normalized = ((ev.value() as f32 / self.max_raw_x) * 2008.0).clamp(0.0, 2007.0);
                                        self.cur_x = normalized as u32;
                                        self.pos_updated = true;
                                    }
                                }
                                AbsoluteAxisType::ABS_MT_POSITION_Y => {
                                    if self.current_slot == 0 {
                                        let normalized = ((ev.value() as f32 / self.max_raw_y) * 60.0).clamp(0.0, 59.0);
                                        self.cur_y = normalized as u32;
                                        self.pos_updated = true;
                                    }
                                }
                                AbsoluteAxisType::ABS_MT_TRACKING_ID => {
                                    if self.current_slot == 0 {
                                        let tracking_id = ev.value();
                                        if tracking_id >= 0 {
                                            self.pending_down = true;
                                        } else {
                                            self.pending_up = true;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        EventType::KEY => {
                            let key = Key::new(ev.code());
                            if key == Key::BTN_TOUCH {
                                if ev.value() == 1 {
                                    self.pending_down = true;
                                } else {
                                    self.pending_up = true;
                                }
                            }
                        }
                        EventType::SYNCHRONIZATION => {
                            if self.pending_down && self.pending_up {
                                // Fast tap completed within single frame
                                self.pending_down = false;
                                self.pending_up = false;
                                self.is_down = false;
                                self.last_reported_x = self.cur_x;
                                self.last_reported_y = self.cur_y;
                                events.push(TouchEvent {
                                    slot: 0,
                                    x: self.cur_x,
                                    y: self.cur_y,
                                    phase: TouchPhase::Down,
                                });
                                events.push(TouchEvent {
                                    slot: 0,
                                    x: self.cur_x,
                                    y: self.cur_y,
                                    phase: TouchPhase::Up,
                                });
                            } else if self.pending_down {
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
                            } else if self.is_down && self.pos_updated {
                                if (self.cur_x as i32 - self.last_reported_x as i32).abs() > 1 
                                    || (self.cur_y as i32 - self.last_reported_y as i32).abs() > 1 
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
