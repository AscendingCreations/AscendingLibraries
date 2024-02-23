use super::Controls;
use glam::{Mat4, Vec3};
#[derive(Clone, Debug, Default)]
pub struct FlatInputs {
    /// move in this direction.
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
}

#[derive(Clone, Debug)]
pub struct FlatSettings {
    pub zoom: f32,
}

impl Default for FlatSettings {
    fn default() -> Self {
        Self { zoom: 1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct FlatControls {
    inputs: FlatInputs,
    settings: FlatSettings,
    view: Mat4,
    eye: Vec3,
    changed: bool,
}

impl FlatControls {
    pub fn inputs(&self) -> &FlatInputs {
        &self.inputs
    }

    pub fn new(settings: FlatSettings) -> Self {
        Self {
            inputs: FlatInputs::default(),
            settings,
            view: Mat4::IDENTITY,
            eye: Vec3::ZERO,
            changed: true,
        }
    }

    pub fn set_inputs(&mut self, inputs: FlatInputs) {
        self.inputs = inputs;
        self.changed = true;
    }
}

impl Controls for FlatControls {
    fn eye(&self) -> [f32; 3] {
        self.eye.into()
    }

    fn update(&mut self, _delta: f32) -> bool {
        let changed = self.changed;

        if changed {
            self.view = Mat4::IDENTITY
                * Mat4::from_scale(Vec3::new(
                    self.settings.zoom,
                    self.settings.zoom,
                    1.0,
                ));
        }

        self.changed = false;
        changed
    }

    fn view(&self) -> mint::ColumnMatrix4<f32> {
        self.view.into()
    }

    fn scale(&self) -> f32 {
        self.settings.zoom
    }
}
