use super::Controls;
use glam::{Mat4, Vec3};

#[derive(Clone, Debug, Default)]
pub struct FirstPersonInputs {
    pub forward: f32,
    pub sideward: f32,
    pub upward: f32,
    pub rotate_x: f32,
    pub rotate_y: f32,
}

#[derive(Clone, Debug)]
pub struct FirstPersonSettings {
    pub sensitivity: f32,
    pub speed: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
}

impl Default for FirstPersonSettings {
    fn default() -> Self {
        Self {
            sensitivity: 1.0,
            speed: 1.0,
            min_pitch: (-89.0_f32).to_radians(),
            max_pitch: (89.0_f32).to_radians(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FirstPersonControls {
    pub inputs: FirstPersonInputs,
    settings: FirstPersonSettings,
    position: Vec3,
    direction: Vec3,
    up: Vec3,
    yaw: f32,
    pitch: f32,
    view: Mat4,
    changed: bool,
}

impl FirstPersonControls {
    pub fn direction(&self) -> [f32; 3] {
        self.direction.into()
    }

    pub fn new(settings: FirstPersonSettings, position: [f32; 3]) -> Self {
        let yaw = (-90.0_f32).to_radians();
        let pitch = 0.0_f32;

        let (yaw_sin, yaw_cos) = yaw.sin_cos();
        let (pitch_sin, pitch_cos) = pitch.sin_cos();

        let direction =
            Vec3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos);

        Self {
            inputs: FirstPersonInputs::default(),
            settings,
            position: position.into(),
            direction,
            up: Vec3::Y,
            yaw,
            pitch,
            view: Mat4::IDENTITY,
            changed: true,
        }
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn position(&self) -> [f32; 3] {
        self.position.into()
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;
        self.update_direction();
    }

    pub fn set_position(&mut self, position: [f32; 3]) {
        self.position = position.into();
        self.changed = true;
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
        self.update_direction();
    }

    fn update_direction(&mut self) {
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();

        self.direction =
            Vec3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos);
        self.changed = true;
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }
}

impl Controls for FirstPersonControls {
    fn eye(&self) -> [f32; 3] {
        self.position.into()
    }

    fn update(&mut self, delta: f32) -> bool {
        let mut changed = self.changed;

        if self.inputs.rotate_x != 0.0 || self.inputs.rotate_y != 0.0 {
            // Update the yaw and pitch.
            self.yaw -=
                self.settings.sensitivity * delta * self.inputs.rotate_x;
            self.pitch +=
                self.settings.sensitivity * delta * self.inputs.rotate_y;

            // Limit the pitch.
            self.pitch = self
                .pitch
                .clamp(self.settings.min_pitch, self.settings.max_pitch);

            // Keep the yaw within [0, 2 * pi[.
            self.yaw = self.yaw.rem_euclid(2.0 * std::f32::consts::PI);

            // Reset the input.
            self.inputs.rotate_x = 0.0;
            self.inputs.rotate_y = 0.0;

            // Calculate the direction vector.
            self.direction = Vec3::new(
                self.yaw.cos() * self.pitch.cos(),
                self.pitch.sin(),
                self.yaw.sin() * self.pitch.cos(),
            );

            changed = true;
        }

        if self.inputs.forward != 0.0 {
            let y = self.position.y;

            let forward = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin());

            self.position +=
                self.settings.speed * delta * self.inputs.forward * forward;
            self.position.y = y;

            // Reset the input.
            self.inputs.forward = 0.0;
            changed = true;
        }

        if self.inputs.sideward != 0.0 {
            let y = self.position.y;

            let sideward =
                Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin()).cross(self.up);

            self.position +=
                self.settings.speed * delta * self.inputs.sideward * sideward;
            self.position.y = y;

            self.inputs.sideward = 0.0;
            changed = true;
        }

        if self.inputs.upward != 0.0 {
            // Move upward or downward.
            self.position.y += self.settings.speed * delta * self.inputs.upward;

            // Reset the input.
            self.inputs.upward = 0.0;
            changed = true;
        }

        if changed {
            // Calculate the view matrix.
            self.view = Mat4::look_at_rh(
                self.position,
                self.position + self.direction,
                self.up,
            );
        }

        self.changed = false;
        changed
    }

    fn view(&self) -> mint::ColumnMatrix4<f32> {
        self.view.into()
    }
}
