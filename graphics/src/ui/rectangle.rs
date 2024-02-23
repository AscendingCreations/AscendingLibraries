use crate::{
    AscendingError, AtlasSet, DrawOrder, DrawType, GpuRenderer, Index,
    OrderedIndex, OtherError, RectVertex, Texture, Vec2, Vec3, Vec4,
};
use cosmic_text::Color;

pub struct Rect {
    pub position: Vec3,
    pub size: Vec2,
    pub color: Color,
    pub image: Option<usize>,
    pub uv: Vec4,
    pub border_width: f32,
    pub border_color: Color,
    pub radius: f32,
    pub use_camera: bool,
    pub store_id: Index,
    pub order: DrawOrder,
    pub render_layer: u32,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Rect {
    pub fn new(renderer: &mut GpuRenderer, render_layer: u32) -> Self {
        Self {
            position: Vec3::default(),
            size: Vec2::default(),
            color: Color::rgba(255, 255, 255, 255),
            image: None,
            uv: Vec4::default(),
            border_width: 0.0,
            border_color: Color::rgba(0, 0, 0, 0),
            radius: 0.0,
            use_camera: false,
            store_id: renderer.new_buffer(),
            order: DrawOrder::default(),
            render_layer,
            changed: true,
        }
    }

    pub fn set_use_camera(&mut self, use_camera: bool) -> &mut Self {
        self.use_camera = use_camera;
        self.changed = true;
        self
    }

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self.changed = true;
        self
    }

    pub fn set_border_color(&mut self, color: Color) -> &mut Self {
        self.border_color = color;
        self.changed = true;
        self
    }

    pub fn set_texture(
        &mut self,
        renderer: &GpuRenderer,
        atlas: &mut AtlasSet,
        path: String,
    ) -> Result<&mut Self, AscendingError> {
        let (id, allocation) =
            Texture::upload_from_with_alloc(path, atlas, renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))?;

        let rect = allocation.rect();

        self.uv = Vec4::new(0.0, 0.0, rect.2 as f32, rect.3 as f32);
        self.image = Some(id);
        self.changed = true;
        Ok(self)
    }

    //Set the Rendering Offset of the container.
    pub fn set_container_uv(&mut self, uv: Vec4) -> &mut Self {
        self.uv = uv;
        self.changed = true;
        self
    }

    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.position = position;
        self.changed = true;
        self
    }

    pub fn set_size(&mut self, size: Vec2) -> &mut Self {
        self.size = size;
        self.changed = true;
        self
    }

    pub fn set_border_width(&mut self, size: f32) -> &mut Self {
        self.border_width = size;
        self.changed = true;
        self
    }

    pub fn set_radius(&mut self, radius: f32) -> &mut Self {
        self.radius = radius;
        self.changed = true;
        self
    }

    pub fn create_quad(
        &mut self,
        renderer: &mut GpuRenderer,
        atlas: &mut AtlasSet,
    ) {
        let (uv, layer) = if let Some(id) = self.image {
            let tex = match atlas.get(id) {
                Some(tex) => tex,
                None => return,
            };
            let (u, v, width, height) = tex.rect();
            (
                [
                    self.uv.x + u as f32,
                    self.uv.y + v as f32,
                    self.uv.z.min(width as f32),
                    self.uv.w.min(height as f32),
                ],
                tex.layer as u32,
            )
        } else {
            ([0.0, 0.0, 0.0, 0.0], 0)
        };

        let buffer = RectVertex {
            position: self.position.to_array(),
            size: self.size.to_array(),
            border_width: self.border_width,
            radius: self.radius,
            uv,
            layer,
            color: self.color.0,
            border_color: self.border_color.0,
            use_camera: u32::from(self.use_camera),
        };

        if let Some(store) = renderer.get_buffer_mut(&self.store_id) {
            store.store = bytemuck::bytes_of(&buffer).to_vec();
            store.changed = true;
        }

        self.order = DrawOrder::new(
            false,
            &self.position,
            1,
            &self.size,
            DrawType::Rectangle,
        );
    }

    /// used to check and update the ShapeVertex array.
    pub fn update(
        &mut self,
        renderer: &mut GpuRenderer,
        atlas: &mut AtlasSet,
    ) -> OrderedIndex {
        // if points added or any data changed recalculate paths.
        if self.changed {
            self.create_quad(renderer, atlas);
            self.changed = false;
        }

        OrderedIndex::new(self.order, self.store_id, 0)
    }

    pub fn check_mouse_bounds(&self, mouse_pos: Vec2) -> bool {
        if self.radius > 0.0 {
            let pos = [self.position.x, self.position.y];

            let inner_size = [
                self.size.x - self.radius * 2.0,
                self.size.y - self.radius * 2.0,
            ];
            let top_left = [pos[0] + self.radius, pos[1] + self.radius];
            let bottom_right =
                [top_left[0] + inner_size[0], top_left[1] + inner_size[1]];

            let top_left_distance =
                [top_left[0] - mouse_pos.x, top_left[1] - mouse_pos.y];
            let bottom_right_distance =
                [mouse_pos.x - bottom_right[0], mouse_pos.y - bottom_right[1]];

            let dist = [
                top_left_distance[0].max(bottom_right_distance[0]).max(0.0),
                top_left_distance[1].max(bottom_right_distance[1]).max(0.0),
            ];

            let dist = (dist[0] * dist[0] + dist[1] * dist[1]).sqrt();

            dist < self.radius
        } else {
            mouse_pos[0] > self.position.x
                && mouse_pos[0] < self.position.x + self.size.x
                && mouse_pos[1] > self.position.y
                && mouse_pos[1] < self.position.y + self.size.y
        }
    }
}
