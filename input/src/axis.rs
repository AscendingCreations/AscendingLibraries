use super::button::Button;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MouseAxis {
    /// The horizontal axis.
    Horizontal,
    /// The vertical axis.
    Vertical,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Axis {
    /// An emulated axis using two buttons where the positive button maps to 1.0 and the negative
    /// button maps to -1.0.
    Emulated { pos: Button, neg: Button },
    /// Mouse motion as an axis.
    MouseMotion {
        axis: MouseAxis,
        limit: bool,
        radius: ordered_float::NotNan<f32>,
    },
    /// Relative mouse motion as an axis.
    RelativeMouseMotion {
        axis: MouseAxis,
        limit: bool,
        radius: ordered_float::NotNan<f32>,
    },
    /// The mouse wheel as an axis.
    MouseWheel { axis: MouseAxis },
}
