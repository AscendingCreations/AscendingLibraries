use super::Controls;
use glam::{Mat4, Vec3};

#[derive(Clone, Debug, Default)]
pub struct FlatInputs {
    pub translation: Vec3,
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

    pub fn inputs_mut(&mut self) -> &mut FlatInputs {
        self.changed = true;
        &mut self.inputs
    }

    pub fn settings(&self) -> &FlatSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut FlatSettings {
        self.changed = true;
        &mut self.settings
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
            self.view = Mat4::from_translation(self.inputs.translation);
        }

        self.changed = false;
        changed
    }

    fn view(&self) -> Mat4 {
        self.view
    }

    fn scale(&self) -> f32 {
        self.settings.zoom
    }
}
