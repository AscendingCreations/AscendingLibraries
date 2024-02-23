use crate::Vec3;

/// This is the location within the World.
/// Height is needed to map world to the correct mouse coords.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WorldBounds {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
    //used to transform world coords into window coords.
    //text rendering can just use anything here.
    pub height: f32,
}

impl WorldBounds {
    pub fn new(
        left: f32,
        bottom: f32,
        right: f32,
        top: f32,
        height: f32,
    ) -> Self {
        Self {
            left,
            bottom,
            right,
            top,
            height,
        }
    }

    pub fn set_offset_within_limits(
        &self,
        offset: &mut Vec3,
        limits: &WorldBounds,
    ) {
        if self.left + offset.x < limits.left {
            offset.x = limits.left - self.left;
        } else if self.right + offset.x > limits.right {
            offset.x = limits.right - self.right;
        }

        if self.bottom + offset.y < limits.bottom {
            offset.y = limits.bottom - self.bottom;
        } else if self.top + offset.y > limits.top {
            offset.y = limits.top - self.top;
        }
    }

    pub fn add_offset(&mut self, offset: Vec3) {
        self.left += offset.x;
        self.right += offset.x;
        self.top += offset.y;
        self.bottom += offset.y;
    }

    pub fn set_within_limits(&mut self, limits: &WorldBounds) {
        if self.left < limits.left {
            self.left = limits.left;
        }

        if self.bottom < limits.bottom {
            self.bottom = limits.bottom;
        }

        if self.top > limits.top {
            self.top = limits.top;
        }

        if self.right > limits.right {
            self.right = limits.right;
        }
    }
}

impl Default for WorldBounds {
    fn default() -> Self {
        Self {
            left: 0.0,
            bottom: 0.0,
            right: 2_147_483_600.0,
            top: 2_147_483_600.0,
            height: 1.0,
        }
    }
}

/// This is the bounds used to clip Text.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bounds {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

impl Bounds {
    pub fn new(left: f32, bottom: f32, right: f32, top: f32) -> Self {
        Self {
            left,
            bottom,
            right,
            top,
        }
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            left: 0.0,
            bottom: 0.0,
            right: 2_147_483_600.0,
            top: 2_147_483_600.0,
        }
    }
}
