use super::Controls;
use glam::{Mat4, Vec3};

#[derive(Clone, Debug, Default)]
pub struct OrbitInputs {
    /// Rotates the camera around the azimuth.
    pub rotate_x: f32,
    /// Rotates the camera around the polar.
    pub rotate_y: f32,
    /// A positive value zooms the camera closer towards the center, while a negative value unzooms
    /// the camera further away from the center.
    pub zoom: f32,
}

#[derive(Clone, Debug)]
pub struct OrbitSettings {
    /// The sensitivity to apply to the rotations.
    pub sensitivity: f32,
    /// The zoom speed.
    pub zoom_speed: f32,
    /// The minimum polar angle.
    pub min_polar: f32,
    /// The maximum polar angle.
    pub max_polar: f32,
    /// The minimum radius.
    pub min_radius: f32,
    /// The maximum radius.
    pub max_radius: f32,
}

impl Default for OrbitSettings {
    fn default() -> Self {
        Self {
            sensitivity: 1.0,
            zoom_speed: 1.0,
            min_polar: (0.0_f32).to_radians(),
            max_polar: (89.0_f32).to_radians(),
            min_radius: 1.0,
            max_radius: 10.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OrbitControls {
    inputs: OrbitInputs,
    settings: OrbitSettings,
    center: Vec3,
    azimuth: f32,
    polar: f32,
    radius: f32,
    view: Mat4,
    eye: Vec3,
    changed: bool,
}

impl OrbitControls {
    pub fn azimuth(&self) -> f32 {
        self.azimuth
    }

    pub fn center(&self) -> [f32; 3] {
        self.center.into()
    }

    pub fn inputs(&self) -> &OrbitInputs {
        &self.inputs
    }

    pub fn new(settings: OrbitSettings, center: [f32; 3], radius: f32) -> Self {
        let radius = radius.clamp(settings.min_radius, settings.max_radius);

        Self {
            inputs: OrbitInputs::default(),
            settings,
            center: center.into(),
            azimuth: 0.0,
            polar: 0.0,
            radius,
            view: Mat4::IDENTITY,
            eye: Vec3::ZERO,
            changed: true,
        }
    }

    pub fn polar(&self) -> f32 {
        self.polar
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn set_aizmuth(&mut self, azimuth: f32) {
        self.azimuth = azimuth.rem_euclid(2.0 * std::f32::consts::PI);
        self.changed = true;
    }

    pub fn set_center(&mut self, center: [f32; 3]) {
        self.center = center.into();
        self.changed = true;
    }

    pub fn set_inputs(&mut self, inputs: OrbitInputs) {
        self.inputs = inputs;
        self.changed = true;
    }

    pub fn set_polar(&mut self, polar: f32) {
        self.polar =
            polar.clamp(self.settings.min_polar, self.settings.max_polar);
        self.changed = true;
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius =
            radius.clamp(self.settings.min_radius, self.settings.max_radius);
        self.changed = true;
    }
}

impl Controls for OrbitControls {
    fn eye(&self) -> [f32; 3] {
        self.eye.into()
    }

    fn update(&mut self, delta: f32) -> bool {
        let mut changed = self.changed;

        if self.inputs.rotate_x != 0.0 || self.inputs.rotate_y != 0.0 {
            // Update the azimuth and polar angle.
            self.azimuth -=
                self.settings.sensitivity * delta * self.inputs.rotate_x;
            self.polar +=
                self.settings.sensitivity * delta * self.inputs.rotate_y;

            // Limit the polar angle.
            self.polar = self
                .polar
                .clamp(self.settings.min_polar, self.settings.max_polar);

            // Keep the azimuth within [0, 2 * PI[.
            self.azimuth = self.azimuth.rem_euclid(2.0 * std::f32::consts::PI);

            // Reset the input.
            self.inputs.rotate_x = 0.0;
            self.inputs.rotate_y = 0.0;
            changed = true;
        }

        if self.inputs.zoom != 0.0 {
            // Update the radius.
            self.radius -= self.settings.zoom_speed * self.inputs.zoom;

            // Limit the radius.
            self.radius = self
                .radius
                .clamp(self.settings.min_radius, self.settings.max_radius);

            // Reset the input.
            self.inputs.zoom = 0.0;
            changed = true;
        }

        if changed {
            // Calculate the eye position.
            let (azimuth_sin, azimuth_cos) = self.azimuth.sin_cos();
            let (polar_sin, polar_cos) = self.polar.sin_cos();

            let x = self.center.x + self.radius * polar_cos * azimuth_cos;
            let y = self.center.y + self.radius * polar_sin;
            let z = self.center.z + self.radius * polar_cos * azimuth_sin;

            self.eye = Vec3::new(x, y, z);

            // Calculate the view matrix.
            let forward = (self.eye - self.center).normalize();
            let sideward = Vec3::Y.cross(forward).normalize();
            let upward = forward.cross(sideward).normalize();

            self.view = Mat4::look_at_rh(self.eye, self.center, upward);
        }

        self.changed = false;
        changed
    }

    fn view(&self) -> mint::ColumnMatrix4<f32> {
        self.view.into()
    }
}
