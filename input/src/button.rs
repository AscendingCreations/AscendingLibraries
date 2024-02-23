use super::Key;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Button {
    // A virtual key on the keyboard.
    Key(Key),
    // A mouse button.
    Mouse(winit::event::MouseButton),
}

impl From<Key> for Button {
    fn from(value: Key) -> Self {
        Button::Key(value)
    }
}

impl From<winit::event::MouseButton> for Button {
    fn from(value: winit::event::MouseButton) -> Self {
        Button::Mouse(value)
    }
}
