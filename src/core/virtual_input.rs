//! Virtual Keyboard Input Device using Linux /dev/uinput
//! Emits real Linux hardware keyboard events (Esc, F1..F12) when virtual buttons are pressed.

use anyhow::Result;
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, EventType, InputEvent, Key};

pub struct VirtualKeyboard {
    device: VirtualDevice,
}

impl VirtualKeyboard {
    pub fn new() -> Result<Self> {
        let mut keys = AttributeSet::<Key>::new();
        keys.insert(Key::KEY_ESC);
        keys.insert(Key::KEY_F1);
        keys.insert(Key::KEY_F2);
        keys.insert(Key::KEY_F3);
        keys.insert(Key::KEY_F4);
        keys.insert(Key::KEY_F5);
        keys.insert(Key::KEY_F6);
        keys.insert(Key::KEY_F7);
        keys.insert(Key::KEY_F8);
        keys.insert(Key::KEY_F9);
        keys.insert(Key::KEY_F10);
        keys.insert(Key::KEY_F11);
        keys.insert(Key::KEY_F12);

        let device = VirtualDeviceBuilder::new()?
            .name("TuxBar Dynamic Function Row")
            .with_keys(&keys)?
            .build()?;

        Ok(Self { device })
    }

    pub fn click_key(&mut self, key: Key) -> Result<()> {
        let down = InputEvent::new(EventType::KEY, key.code(), 1);
        let syn1 = InputEvent::new(EventType::SYNCHRONIZATION, 0, 0);
        let up = InputEvent::new(EventType::KEY, key.code(), 0);
        let syn2 = InputEvent::new(EventType::SYNCHRONIZATION, 0, 0);
        self.device.emit(&[down, syn1, up, syn2])?;
        Ok(())
    }
}
