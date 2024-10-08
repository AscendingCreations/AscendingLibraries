use crate::{
    CameraType, DrawOrder, GpuRenderer, GraphicsError, Index, Mesh2DVertex,
    OrderedIndex, OtherError, Vec2, Vec3, Vec4, VertexBuilder,
};
use cosmic_text::Color;
use lyon::{
    lyon_tessellation::{FillOptions, StrokeOptions},
    math::Point as LPoint,
    path::Polygon,
    tessellation as tess,
};

/// Mode in how we will Create the Mesh's vertex layout.
#[derive(Debug, Copy, Clone)]
pub enum DrawMode {
    /// This will mostly only create lines without filling.
    Stroke(StrokeOptions),
    /// This will Create a filled area.
    Fill(FillOptions),
}

impl DrawMode {
    pub fn stroke(width: f32) -> DrawMode {
        DrawMode::Stroke(StrokeOptions::default().with_line_width(width))
    }

    pub fn fill() -> DrawMode {
        DrawMode::Fill(FillOptions::default())
    }
}

/// 2D Meshs to render to screen.
///
pub struct Mesh2D {
    /// Position on the Screen.
    pub position: Vec3,
    /// Width and Height of the mesh.
    pub size: Vec2,
    /// Color of the Mesh.
    pub color: Color,
    /// Saved Verticies of the Mesh.
    pub vertices: Vec<Mesh2DVertex>,
    /// Saved indices of the mesh.
    pub indices: Vec<u32>,
    /// Mesh's Vertex Buffer Store [`Index`].
    pub vbo_store_id: Index,
    /// the draw order of the rect. created/updated when update is called.
    pub order: DrawOrder,
    /// Rendering Layer of the rect used in DrawOrder.
    pub render_layer: u32,
    /// Index Max Generated by the Mesh Builder.
    pub high_index: u32,
    /// Overides the absolute order values based on position.
    pub order_override: Option<Vec3>,
    // if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Mesh2D {
    /// Creates a new [`Mesh2D`] with rendering layer.
    ///
    pub fn new(renderer: &mut GpuRenderer, render_layer: u32) -> Self {
        Self {
            position: Vec3::default(),
            size: Vec2::default(),
            color: Color::rgba(255, 255, 255, 255),
            vbo_store_id: renderer.default_buffer(),
            order: DrawOrder::default(),
            changed: true,
            vertices: Vec::new(),
            indices: Vec::new(),
            high_index: 0,
            render_layer,
            order_override: None,
        }
    }

    /// Unloads the [`Mesh2D`] from the Instance Buffers Store.
    ///
    pub fn unload(&self, renderer: &mut GpuRenderer) {
        renderer.remove_buffer(self.vbo_store_id);
    }

    /// Updates the [`Mesh2D`]'s order_override.
    ///
    pub fn set_order_override(
        &mut self,
        order_override: Option<Vec3>,
    ) -> &mut Self {
        self.changed = true;
        self.order_override = order_override;
        self
    }

    /// Appends Mesh's from the [`Mesh2DBuilder`] into the [`Mesh2D`].
    ///
    pub fn from_builder(&mut self, builder: Mesh2DBuilder) {
        self.position =
            Vec3::new(builder.bounds.x, builder.bounds.y, builder.z);
        self.size = Vec2::new(
            builder.bounds.z - builder.bounds.x,
            builder.bounds.w - builder.bounds.y,
        );
        self.vertices.extend_from_slice(&builder.buffer.vertices);
        self.indices.extend_from_slice(&builder.buffer.indices);
        self.high_index = builder.high_index;
    }

    /// Sets the [`Mesh2D`]'s [`Color`].
    ///
    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self.changed = true;
        self
    }

    /// Sets the [`Mesh2D`]'s Position.
    ///
    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.position = position;
        self.changed = true;
        self
    }

    /// Sets the [`Mesh2D`]'s width and height.
    ///
    pub fn set_size(&mut self, size: Vec2) -> &mut Self {
        self.size = size;
        self.changed = true;
        self
    }

    /// Updates the [`Mesh2D`]'s Buffers to prepare them for rendering.
    ///
    pub fn create_quad(&mut self, renderer: &mut GpuRenderer) {
        if let Some(store) = renderer.get_buffer_mut(self.vbo_store_id) {
            let vertex_bytes: &[u8] = bytemuck::cast_slice(&self.vertices);
            let index_bytes: &[u8] = bytemuck::cast_slice(&self.indices);
            store.store.resize_with(vertex_bytes.len(), || 0);
            store.indexs.resize_with(index_bytes.len(), || 0);
            store.store.copy_from_slice(vertex_bytes);
            store.indexs.copy_from_slice(index_bytes);
            store.changed = true;
        }

        let order_pos = match self.order_override {
            Some(o) => o,
            None => self.position,
        };

        self.order =
            DrawOrder::new(self.color.a() < 255, &order_pos, self.render_layer);
    }

    /// Used to check and update the vertex array.
    /// Returns a [`OrderedIndex`] used in Rendering.
    ///
    pub fn update(&mut self, renderer: &mut GpuRenderer) -> OrderedIndex {
        if self.changed {
            self.create_quad(renderer);
            self.changed = false;
        }

        OrderedIndex::new(self.order, self.vbo_store_id, self.high_index)
    }

    /// Checks if Mouse position is within the [`Mesh2D`]'s Bounds.
    pub fn check_mouse_bounds(&self, mouse_pos: Vec2) -> bool {
        mouse_pos[0] > self.position.x
            && mouse_pos[0] < self.position.x + self.size.x
            && mouse_pos[1] > self.position.y
            && mouse_pos[1] < self.position.y + self.size.y
    }
}

/// [`Mesh2D`] based on ggez Meshbuilder.
///
#[derive(Debug, Clone)]
pub struct Mesh2DBuilder {
    buffer: tess::geometry_builder::VertexBuffers<Mesh2DVertex, u32>,
    bounds: Vec4,
    z: f32,
    high_index: u32,
    camera_type: CameraType,
}

impl Default for Mesh2DBuilder {
    fn default() -> Self {
        Self {
            buffer: tess::VertexBuffers::new(),
            bounds: Vec4::new(0.0, 0.0, 0.0, 0.0),
            z: 1.0,
            high_index: 0,
            camera_type: CameraType::None,
        }
    }
}

impl Mesh2DBuilder {
    /// Creates a new [`Mesh2DBuilder`] with [`CameraType`].
    ///
    pub fn with_camera(camera_type: CameraType) -> Self {
        Self {
            camera_type,
            ..Self::default()
        }
    }

    /// Finalizes the [`Mesh2DBuilder`] so it can be appended to a [`Mesh2D`].
    ///
    pub fn finalize(mut self) -> Self {
        let [minx, miny, maxx, maxy, minz] = self.buffer.vertices.iter().fold(
            [f32::MAX, f32::MAX, f32::MIN, f32::MIN, 1.0],
            |[minx, miny, maxx, maxy, minz], vert| {
                let [x, y, z] = vert.position;
                [
                    minx.min(x),
                    miny.min(y),
                    maxx.max(x),
                    maxy.max(y),
                    minz.min(z),
                ]
            },
        );

        let high_index = self
            .buffer
            .indices
            .iter()
            .fold(0, |max, index| max.max(*index));
        self.bounds = Vec4::new(minx, miny, maxx, maxy);
        self.z = minz;
        self.high_index = high_index;
        self
    }

    /// Draws a Line within the [`Mesh2DBuilder`] vertex buffer.
    ///
    pub fn line(
        &mut self,
        points: &[Vec2],
        z: f32,
        width: f32,
        color: Color,
    ) -> Result<&mut Self, GraphicsError> {
        self.polyline(DrawMode::stroke(width), points, z, color)
    }

    /// Draws a Circle within the [`Mesh2DBuilder`] vertex buffer.
    ///
    pub fn circle(
        &mut self,
        mode: DrawMode,
        point: Vec2,
        radius: f32,
        tolerance: f32,
        z: f32,
        color: Color,
    ) -> Result<&mut Self, GraphicsError> {
        assert!(tolerance > 0.0, "Tolerances <= 0 are invalid");
        {
            let buffers = &mut self.buffer;
            let vb = VertexBuilder {
                z,
                color,
                camera: self.camera_type as u32,
            };
            match mode {
                DrawMode::Fill(fill_options) => {
                    let mut tessellator = tess::FillTessellator::new();
                    tessellator.tessellate_circle(
                        tess::math::point(point.x, point.y),
                        radius,
                        &fill_options.with_tolerance(tolerance),
                        &mut tess::BuffersBuilder::new(buffers, vb),
                    )?;
                }
                DrawMode::Stroke(options) => {
                    let mut tessellator = tess::StrokeTessellator::new();
                    tessellator.tessellate_circle(
                        tess::math::point(point.x, point.y),
                        radius,
                        &options.with_tolerance(tolerance),
                        &mut tess::BuffersBuilder::new(buffers, vb),
                    )?;
                }
            };
        }
        Ok(self)
    }

    /// Draws an Ellipse within the [`Mesh2DBuilder`] vertex buffer.
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn ellipse(
        &mut self,
        mode: DrawMode,
        point: Vec2,
        radius1: f32,
        radius2: f32,
        tolerance: f32,
        z: f32,
        color: Color,
    ) -> Result<&mut Self, GraphicsError> {
        assert!(tolerance > 0.0, "Tolerances <= 0 are invalid");
        {
            let buffers = &mut self.buffer;
            let vb = VertexBuilder {
                z,
                color,
                camera: self.camera_type as u32,
            };
            match mode {
                DrawMode::Fill(fill_options) => {
                    let builder = &mut tess::BuffersBuilder::new(buffers, vb);
                    let mut tessellator = tess::FillTessellator::new();
                    tessellator.tessellate_ellipse(
                        tess::math::point(point.x, point.y),
                        tess::math::vector(radius1, radius2),
                        tess::math::Angle { radians: 0.0 },
                        tess::path::Winding::Positive,
                        &fill_options.with_tolerance(tolerance),
                        builder,
                    )?;
                }
                DrawMode::Stroke(options) => {
                    let builder = &mut tess::BuffersBuilder::new(buffers, vb);
                    let mut tessellator = tess::StrokeTessellator::new();
                    tessellator.tessellate_ellipse(
                        tess::math::point(point.x, point.y),
                        tess::math::vector(radius1, radius2),
                        tess::math::Angle { radians: 0.0 },
                        tess::path::Winding::Positive,
                        &options.with_tolerance(tolerance),
                        builder,
                    )?;
                }
            };
        }
        Ok(self)
    }

    /// Draws an Polyline within the [`Mesh2DBuilder`] vertex buffer.
    ///
    pub fn polyline(
        &mut self,
        mode: DrawMode,
        points: &[Vec2],
        z: f32,
        color: Color,
    ) -> Result<&mut Self, GraphicsError> {
        if points.len() < 2 {
            return Err(GraphicsError::Other(OtherError::new(
                "MeshBuilder::polyline() got a list of < 2 points",
            )));
        }

        self.polyline_inner(mode, points, false, z, color)
    }

    /// Draws an Polygon within the [`Mesh2DBuilder`] vertex buffer.
    ///
    pub fn polygon(
        &mut self,
        mode: DrawMode,
        points: &[Vec2],
        z: f32,
        color: Color,
    ) -> Result<&mut Self, GraphicsError> {
        if points.len() < 3 {
            return Err(GraphicsError::Other(OtherError::new(
                "MeshBuilder::polygon() got a list of < 3 points",
            )));
        }

        self.polyline_inner(mode, points, true, z, color)
    }

    fn polyline_inner(
        &mut self,
        mode: DrawMode,
        points: &[Vec2],
        is_closed: bool,
        z: f32,
        color: Color,
    ) -> Result<&mut Self, GraphicsError> {
        let vb = VertexBuilder {
            z,
            color,
            camera: self.camera_type as u32,
        };
        self.polyline_with_vertex_builder(mode, points, is_closed, vb)
    }

    /// Draws an Polyline within the [`Mesh2DBuilder`] vertex buffer using a custom vertex builder.
    ///
    pub fn polyline_with_vertex_builder<V>(
        &mut self,
        mode: DrawMode,
        points: &[Vec2],
        is_closed: bool,
        vb: V,
    ) -> Result<&mut Self, GraphicsError>
    where
        V: tess::StrokeVertexConstructor<Mesh2DVertex>
            + tess::FillVertexConstructor<Mesh2DVertex>,
    {
        {
            assert!(points.len() > 1);
            let buffers = &mut self.buffer;
            let points: Vec<LPoint> = points
                .iter()
                .cloned()
                .map(|p| tess::math::point(p.x, p.y))
                .collect();
            let polygon = Polygon {
                points: &points,
                closed: is_closed,
            };
            match mode {
                DrawMode::Fill(options) => {
                    let builder = &mut tess::BuffersBuilder::new(buffers, vb);
                    let tessellator = &mut tess::FillTessellator::new();
                    tessellator
                        .tessellate_polygon(polygon, &options, builder)?;
                }
                DrawMode::Stroke(options) => {
                    let builder = &mut tess::BuffersBuilder::new(buffers, vb);
                    let tessellator = &mut tess::StrokeTessellator::new();
                    tessellator
                        .tessellate_polygon(polygon, &options, builder)?;
                }
            };
        }
        Ok(self)
    }

    /// Draws an Rectangle within the [`Mesh2DBuilder`] vertex buffer.
    ///
    pub fn rectangle(
        &mut self,
        mode: DrawMode,
        bounds: Vec4,
        z: f32,
        color: Color,
    ) -> Result<&mut Self, GraphicsError> {
        {
            let buffers = &mut self.buffer;
            let rect = tess::math::Box2D::from_origin_and_size(
                tess::math::point(bounds.x, bounds.y),
                tess::math::size(bounds.z, bounds.w),
            );
            let vb = VertexBuilder {
                z,
                color,
                camera: self.camera_type as u32,
            };
            match mode {
                DrawMode::Fill(fill_options) => {
                    let builder = &mut tess::BuffersBuilder::new(buffers, vb);
                    let mut tessellator = tess::FillTessellator::new();
                    tessellator.tessellate_rectangle(
                        &rect,
                        &fill_options,
                        builder,
                    )?;
                }
                DrawMode::Stroke(options) => {
                    let builder = &mut tess::BuffersBuilder::new(buffers, vb);
                    let mut tessellator = tess::StrokeTessellator::new();
                    tessellator
                        .tessellate_rectangle(&rect, &options, builder)?;
                }
            };
        }
        Ok(self)
    }

    /// Draws an Rounded Rectangle within the [`Mesh2DBuilder`] vertex buffer.
    ///
    pub fn rounded_rectangle(
        &mut self,
        mode: DrawMode,
        bounds: Vec4,
        z: f32,
        radius: f32,
        color: Color,
    ) -> Result<&mut Self, GraphicsError> {
        {
            let buffers = &mut self.buffer;
            let rect = tess::math::Box2D::from_origin_and_size(
                tess::math::point(bounds.x, bounds.y),
                tess::math::size(bounds.z, bounds.w),
            );
            let radii = tess::path::builder::BorderRadii::new(radius);
            let vb = VertexBuilder {
                z,
                color,
                camera: self.camera_type as u32,
            };
            let mut path_builder = tess::path::Path::builder();
            path_builder.add_rounded_rectangle(
                &rect,
                &radii,
                tess::path::Winding::Positive,
            );
            let path = path_builder.build();

            match mode {
                DrawMode::Fill(fill_options) => {
                    let builder = &mut tess::BuffersBuilder::new(buffers, vb);
                    let mut tessellator = tess::FillTessellator::new();
                    tessellator.tessellate_path(
                        &path,
                        &fill_options,
                        builder,
                    )?;
                }
                DrawMode::Stroke(options) => {
                    let builder = &mut tess::BuffersBuilder::new(buffers, vb);
                    let mut tessellator = tess::StrokeTessellator::new();
                    tessellator.tessellate_path(&path, &options, builder)?;
                }
            };
        }
        Ok(self)
    }

    /// Draws an Triangle within the [`Mesh2DBuilder`] vertex buffer.
    ///
    pub fn triangles(
        &mut self,
        triangles: &[Vec2],
        z: f32,
        color: Color,
    ) -> Result<&mut Self, GraphicsError> {
        {
            if (triangles.len() % 3) != 0 {
                return Err(GraphicsError::Other(OtherError::new(
                    "Called MeshBuilder::triangles() with points that have a length not a multiple of 3.",
                )));
            }
            let tris = triangles
                .iter()
                .cloned()
                .map(|p| lyon::math::point(p.x, p.y))
                .collect::<Vec<_>>();
            let tris = tris.chunks(3);
            let vb = VertexBuilder {
                z,
                color,
                camera: self.camera_type as u32,
            };
            for tri in tris {
                assert!(tri.len() == 3);
                let first_index: u32 =
                    self.buffer.vertices.len().try_into().unwrap();
                self.buffer.vertices.push(vb.new_vertex(tri[0]));
                self.buffer.vertices.push(vb.new_vertex(tri[1]));
                self.buffer.vertices.push(vb.new_vertex(tri[2]));
                self.buffer.indices.push(first_index);
                self.buffer.indices.push(first_index + 1);
                self.buffer.indices.push(first_index + 2);
            }
        }
        Ok(self)
    }
}
