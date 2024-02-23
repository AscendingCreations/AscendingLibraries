//! Identify keyboard keys.
use serde::{Deserialize, Serialize};
pub use winit::keyboard::KeyLocation as Location;
pub use winit::keyboard::NamedKey as Named;
/// A key on the keyboard.
/// Used to convert smolStr into Char for Direct usage as example: Character('v')
/// [`winit`]: https://docs.rs/winit/0.29.10/winit/keyboard/enum.Key.html
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deserialize,
    Serialize,
)]
pub enum Key {
    /// A key with an established name.
    Named(Named),

    /// A key string that corresponds to the character typed by the user, taking into account the
    /// user’s current locale setting, and any system-level keyboard mapping overrides that are in
    /// effect.
    Character(char),
}
