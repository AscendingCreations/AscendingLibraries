mod first_person;
mod flat;
mod flying;
mod orbit;

pub trait Controls {
    /// Retrieves the eye position.
    fn eye(&self) -> [f32; 3];

    /// Processes the inputs and recalculates the view matrix and eye position if the state
    /// changed. Returns `true` if anything was updated, otherwise returns `false`.
    fn update(&mut self, delta: f32) -> bool;

    /// Retrieves the view matrix.
    fn view(&self) -> mint::ColumnMatrix4<f32>;

    ///returns the scale if one exists. otherwise 1.0
    fn scale(&self) -> f32 {
        1.0
    }
}

pub use first_person::{
    FirstPersonControls, FirstPersonInputs, FirstPersonSettings,
};
pub use flat::{FlatControls, FlatInputs, FlatSettings};
pub use flying::{FlyingControls, FlyingInputs, FlyingSettings};
pub use orbit::{OrbitControls, OrbitInputs, OrbitSettings};
