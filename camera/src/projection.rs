use glam::Mat4;

#[derive(Clone, Copy, Debug)]
pub enum Projection {
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
    Perspective {
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    },
}

impl From<Projection> for Mat4 {
    fn from(proj: Projection) -> Mat4 {
        match proj {
            Projection::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => Mat4::orthographic_rh(left, right, bottom, top, near, far),
            Projection::Perspective {
                fov,
                aspect_ratio,
                near,
                far,
            } => Mat4::perspective_rh_gl(fov, aspect_ratio, near, far),
        }
    }
}

impl From<Projection> for mint::ColumnMatrix4<f32> {
    fn from(proj: Projection) -> mint::ColumnMatrix4<f32> {
        let matrix: Mat4 = proj.into();

        matrix.into()
    }
}
