//! Touch Bar digitizer input listener using Linux evdev subsystem

use crate::types::{TouchEvent, TouchPhase};
use anyhow::Result;
use evdev::{AbsoluteAxisType, Device, EventType, Key};
use std::path::Path;

pub struct TouchListener {
    device: Device,
    max_raw_x: f32,
    max_raw_y: f32,
    last_x: u32,
    last_y: u32,
}

impl TouchListener {
    pub fn open(path: &Path) -> Result<Self> {
        let device = Device::open(path)?;

        // MacBookPro16,1 Touch Bar digitizer hardware boundaries
        let max_x = 2170.0f32;
        let max_y = 60.0f32;

        Ok(Self {
            device,
            max_raw_x: max_x,
            max_raw_y: max_y,
            last_x: 0,
            last_y: 0,
        })
    }

    pub fn poll_events(&mut self) -> Result<Vec<TouchEvent>> {
        let mut events = Vec::new();

        for ev in self.device.fetch_events()? {
            match ev.event_type() {
                EventType::ABSOLUTE => {
                    let code = AbsoluteAxisType(ev.code());
                    if code == AbsoluteAxisType::ABS_MT_POSITION_X {
                        let normalized = ((ev.value() as f32 / self.max_raw_x) * 2008.0).clamp(0.0, 2007.0);
                        self.last_x = normalized as u32;
                    } else if code == AbsoluteAxisType::ABS_MT_POSITION_Y {
                        let normalized = ((ev.value() as f32 / self.max_raw_y) * 60.0).clamp(0.0, 59.0);
                        self.last_y = normalized as u32;
                    }
                }
                EventType::KEY => {
                    let key = Key::new(ev.code());
                    if key == Key::BTN_TOUCH {
                        let phase = if ev.value() == 1 {
                            TouchPhase::Down
                        } else {
                            TouchPhase::Up
                        };
                        events.push(TouchEvent {
                            slot: 0,
                            x: self.last_x,
                            y: self.last_y,
                            phase,
                        });
                    }
                }
                _ => {}
            }
        }

        Ok(events)
    }
}
