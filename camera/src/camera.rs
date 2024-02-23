use super::projection::Projection;

#[derive(Clone, Debug)]
pub struct Camera<Controls>
where
    Controls: super::controls::Controls,
{
    projection: Projection,
    controls: Controls,
    changed: bool,
}

impl<Controls> Camera<Controls>
where
    Controls: super::controls::Controls,
{
    pub fn controls(&self) -> &Controls {
        &self.controls
    }

    pub fn controls_mut(&mut self) -> &mut Controls {
        &mut self.controls
    }

    pub fn eye(&self) -> [f32; 3] {
        self.controls.eye()
    }

    pub fn new(projection: Projection, controls: Controls) -> Self {
        Self {
            projection,
            controls,
            changed: true,
        }
    }

    pub fn projection(&self) -> mint::ColumnMatrix4<f32> {
        self.projection.into()
    }

    pub fn set_controls(&mut self, controls: Controls) -> Controls {
        let controls = std::mem::replace(&mut self.controls, controls);
        self.changed = true;
        controls
    }

    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
        self.changed = true;
    }

    pub fn update(&mut self, delta: f32) -> bool {
        let mut changed = self.changed;

        changed |= self.controls.update(delta);

        self.changed = false;
        changed
    }

    pub fn view(&self) -> mint::ColumnMatrix4<f32> {
        self.controls.view()
    }

    pub fn scale(&self) -> f32 {
        self.controls.scale()
    }
}
