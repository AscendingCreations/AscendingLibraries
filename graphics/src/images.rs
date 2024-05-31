mod pipeline;
mod render;
mod vertex;

pub use pipeline::*;
pub use render::*;
pub use vertex::*;

use crate::{
    AtlasSet, Bounds, CameraType, Color, DrawOrder, FlipStyle,
    GpuRenderer, Index, OrderedIndex, Vec2, Vec3, Vec4,
};

/// Basic and Fast Image Rendering Type. Best used for Sprites and Objects in the world.
pub struct Image {
    /// Position of the object
    pub pos: Vec3,
    /// Height and Width
    pub hw: Vec2,
    /// Static texture offsets or animation frame positions
    pub uv: Vec4,
    /// Color.
    pub color: Color,
    /// frames, frames_per_row: this will cycle thru
    /// frames per row at the uv start.
    pub frames: Vec2,
    /// in millsecs 1000 = 1sec
    pub switch_time: u32,
    /// turn on animation if set.
    pub animate: bool,
    /// Global Camera the Shader will use to render the object with
    pub camera_type: CameraType,
    /// Texture area location in Atlas.
    pub texture: Option<usize>,
    /// Buffer's store Index.
    pub store_id: Index,
    /// Ordering Type, used to order the Stores in the buffers.
    pub order: DrawOrder,
    /// Layer this type is rendering on.
    pub render_layer: u32,
    /// Clip bounds if enabled in the renderer.
    pub bounds: Option<Bounds>,
    /// Directional Flip.
    pub flip_style: FlipStyle,
    /// direct angle of rotation from the center Axis.
    pub rotation_angle: f32,
    /// When true tells system to update the buffers.
    pub changed: bool,
}

impl Image {
    pub fn new(
        texture: Option<usize>,
        renderer: &mut GpuRenderer,
        render_layer: u32,
    ) -> Self {
        Self {
            pos: Vec3::default(),
            hw: Vec2::default(),
            uv: Vec4::default(),
            frames: Vec2::default(),
            switch_time: 0,
            animate: false,
            camera_type: CameraType::None,
            color: Color::rgba(255, 255, 255, 255),
            texture,
            store_id: renderer.new_buffer(
                bytemuck::bytes_of(&ImageVertex::default()).len(),
                0,
            ),
            order: DrawOrder::default(),
            render_layer,
            bounds: None,
            flip_style: FlipStyle::None,
            rotation_angle: 0.0,
            changed: true,
        }
    }

    pub fn unload(&self, renderer: &mut GpuRenderer) {
        renderer.remove_buffer(self.store_id);
    }

    pub fn update_bounds(&mut self, bounds: Option<Bounds>) -> &mut Self {
        self.bounds = bounds;
        self
    }

    pub fn set_flip_style(&mut self, flip_style: FlipStyle) -> &mut Self {
        self.changed = true;
        self.flip_style = flip_style;
        self
    }

    pub fn set_rotation_angle(&mut self, rotation_angle: f32) -> &mut Self {
        self.changed = true;
        self.rotation_angle = rotation_angle;
        self
    }

    pub fn set_pos(&mut self, pos: Vec3) -> &mut Self {
        self.changed = true;
        self.pos = pos;
        self
    }

    pub fn set_size(&mut self, hw: Vec2) -> &mut Self {
        self.changed = true;
        self.hw = hw;
        self
    }

    pub fn set_frames(&mut self, frames: Vec2) -> &mut Self {
        self.changed = true;
        self.frames = frames;
        self
    }

    pub fn set_animate(&mut self, animate: bool) -> &mut Self {
        self.changed = true;
        self.animate = animate;
        self
    }

    pub fn set_uv(&mut self, uv: Vec4) -> &mut Self {
        self.changed = true;
        self.uv = uv;
        self
    }

    pub fn set_render_layer(&mut self, render_layer: u32) -> &mut Self {
        self.changed = true;
        self.render_layer = render_layer;
        self
    }

    pub fn set_texture(&mut self, texture: Option<usize>) -> &mut Self {
        self.changed = true;
        self.texture = texture;
        self
    }

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.changed = true;
        self.color = color;
        self
    }

    pub fn set_camera_type(&mut self, camera_type: CameraType) -> &mut Self {
        self.changed = true;
        self.camera_type = camera_type;
        self
    }

    pub fn set_switch_time(&mut self, switch_time: u32) -> &mut Self {
        self.changed = true;
        self.switch_time = switch_time;
        self
    }

    fn create_quad(
        &mut self,
        renderer: &mut GpuRenderer,
        atlas: &mut AtlasSet,
    ) {
        let allocation = match &self.texture {
            Some(id) => {
                if let Some(allocation) = atlas.get(*id) {
                    allocation
                } else {
                    return;
                }
            }
            None => return,
        };

        let (u, v, width, height) = allocation.rect();
        let tex_data = (
            self.uv.x + u as f32,
            self.uv.y + v as f32,
            self.uv.z.min(width as f32),
            self.uv.w.min(height as f32),
        );

        let instance = ImageVertex {
            position: self.pos.to_array(),
            hw: self.hw.to_array(),
            tex_data: tex_data.into(),
            color: self.color.0,
            frames: self.frames.to_array(),
            animate: u32::from(self.animate),
            camera_type: self.camera_type as u32,
            time: self.switch_time,
            layer: allocation.layer as i32,
            flip_style: self.flip_style as u32,
            angle: self.rotation_angle,
        };

        if let Some(store) = renderer.get_buffer_mut(self.store_id) {
            let bytes = bytemuck::bytes_of(&instance);
            store.store.resize_with(bytes.len(), || 0);
            store.store.copy_from_slice(bytes);
            store.changed = true;
        }

        self.order =
            DrawOrder::new(self.color.a() < 255, &self.pos, self.render_layer);
        self.changed = false;
    }

    /// Updates the buffer if changed is true.
    /// Returns OrderIndex used by the renderer to position
    /// and upload obejcts to the Instance Buffer Arrays on the GPU.
    pub fn update(
        &mut self,
        renderer: &mut GpuRenderer,
        atlas: &mut AtlasSet,
    ) -> OrderedIndex {
        // if pos or tex_pos or color changed.
        if self.changed {
            self.create_quad(renderer, atlas);
        }

        OrderedIndex::new_with_bounds(
            self.order,
            self.store_id,
            0,
            self.bounds,
            self.camera_type,
        )
    }
}
