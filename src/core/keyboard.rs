//! Keyboard listener to intercept physical Fn key on Apple MacBooks

use anyhow::Result;
use evdev::{Device, EventType, Key};
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct KeyboardListener {
    device: Device,
}

impl KeyboardListener {
    pub fn open(path: &Path) -> Result<Self> {
        let device = Device::open(path)?;
        unsafe {
            let fd = device.as_raw_fd();
            let flags = libc::fcntl(fd, libc::F_GETFL, 0);
            libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
        Ok(Self { device })
    }

    /// Automatically find the physical Apple Internal Keyboard among /dev/input/event*
    pub fn find_apple_keyboard() -> Option<Self> {
        for (path, dev) in evdev::enumerate() {
            let name = dev.name().unwrap_or("").to_lowercase();
            if name.contains("apple internal keyboard") {
                if let Some(keys) = dev.supported_keys() {
                    if keys.contains(Key::KEY_FN) {
                        if let Ok(listener) = Self::open(&path) {
                            println!("[TuxBar] Found Apple Internal Keyboard with Fn key at {:?}", path);
                            return Some(listener);
                        }
                    }
                }
            }
        }
        None
    }

    /// Poll for Fn key state change:
    /// Some(true) = Fn pressed down
    /// Some(false) = Fn released
    /// None = no change
    pub fn poll_fn_key(&mut self) -> Option<bool> {
        if let Ok(events) = self.device.fetch_events() {
            for ev in events {
                if ev.event_type() == EventType::KEY {
                    let key = Key::new(ev.code());
                    if key == Key::KEY_FN {
                        return Some(ev.value() == 1);
                    }
                }
            }
        }
        None
    }
}
