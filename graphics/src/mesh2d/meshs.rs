use crate::{
    AscendingError, BufferLayout, DrawOrder, DrawType, GpuRenderer, Index,
    Mesh2DVertex, OrderedIndex, OtherError, Vec2, Vec3, Vec4, VertexBuilder,
};
use cosmic_text::Color;
use lyon::{
    lyon_tessellation::{FillOptions, StrokeOptions},
    math::Point as LPoint,
    path::Polygon,
    tessellation as tess,
};

#[derive(Debug, Copy, Clone)]
pub enum DrawMode {
    Stroke(StrokeOptions),
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

pub struct Mesh2D {
    pub position: Vec3,
    pub size: Vec2,
    pub color: Color,
    pub vertices: Vec<Mesh2DVertex>,
    pub indices: Vec<u32>,
    pub vbo_store_id: Index,
    pub order: DrawOrder,
    pub high_index: u32,
    // if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Mesh2D {
    pub fn new(renderer: &mut GpuRenderer) -> Self {
        Self {
            position: Vec3::default(),
            size: Vec2::default(),
            color: Color::rgba(255, 255, 255, 255),
            vbo_store_id: renderer.new_buffer(),
            order: DrawOrder::default(),
            changed: true,
            vertices: Vec::new(),
            indices: Vec::new(),
            high_index: 0,
        }
    }

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

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
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

    pub fn create_quad(&mut self, renderer: &mut GpuRenderer) {
        if let Some(store) = renderer.get_buffer_mut(&self.vbo_store_id) {
            let mut vertex_bytes = Vec::with_capacity(
                self.vertices.len() * Mesh2DVertex::stride(),
            );
            let mut index_bytes = Vec::with_capacity(self.indices.len() * 4);

            for vertex in &self.vertices {
                vertex_bytes.append(&mut bytemuck::bytes_of(vertex).to_vec());
            }

            for index in &self.indices {
                index_bytes.append(&mut bytemuck::bytes_of(index).to_vec());
            }

            store.store = vertex_bytes;
            store.indexs = index_bytes;
            store.changed = true;
        }

        self.order = DrawOrder::new(
            false,
            &self.position,
            1,
            &self.size,
            DrawType::Mesh2D,
        );
    }

    // used to check and update the ShapeVertex array.
    pub fn update(&mut self, renderer: &mut GpuRenderer) -> OrderedIndex {
        // if points added or any data changed recalculate paths.
        if self.changed {
            self.create_quad(renderer);
            self.changed = false;
        }

        OrderedIndex::new(self.order, self.vbo_store_id, self.high_index)
    }

    pub fn check_mouse_bounds(&self, mouse_pos: Vec2) -> bool {
        mouse_pos[0] > self.position.x
            && mouse_pos[0] < self.position.x + self.size.x
            && mouse_pos[1] > self.position.y
            && mouse_pos[1] < self.position.y + self.size.y
    }
}

//MeshBuilder based on ggez Meshbuilder.
#[derive(Debug, Clone)]
pub struct Mesh2DBuilder {
    buffer: tess::geometry_builder::VertexBuffers<Mesh2DVertex, u32>,
    bounds: Vec4,
    z: f32,
    high_index: u32,
    use_camera: bool,
}

impl Default for Mesh2DBuilder {
    fn default() -> Self {
        Self {
            buffer: tess::VertexBuffers::new(),
            bounds: Vec4::new(0.0, 0.0, 0.0, 0.0),
            z: 1.0,
            high_index: 0,
            use_camera: false,
        }
    }
}

impl Mesh2DBuilder {
    pub fn with_camera() -> Self {
        Self {
            use_camera: true,
            ..Self::default()
        }
    }

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

    pub fn line(
        &mut self,
        points: &[Vec2],
        z: f32,
        width: f32,
        color: Color,
    ) -> Result<&mut Self, AscendingError> {
        self.polyline(DrawMode::stroke(width), points, z, color)
    }

    pub fn circle(
        &mut self,
        mode: DrawMode,
        point: Vec2,
        radius: f32,
        tolerance: f32,
        z: f32,
        color: Color,
    ) -> Result<&mut Self, AscendingError> {
        assert!(tolerance > 0.0, "Tolerances <= 0 are invalid");
        {
            let buffers = &mut self.buffer;
            let vb = VertexBuilder {
                z,
                color,
                camera: self.use_camera,
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
    ) -> Result<&mut Self, AscendingError> {
        assert!(tolerance > 0.0, "Tolerances <= 0 are invalid");
        {
            let buffers = &mut self.buffer;
            let vb = VertexBuilder {
                z,
                color,
                camera: self.use_camera,
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

    pub fn polyline(
        &mut self,
        mode: DrawMode,
        points: &[Vec2],
        z: f32,
        color: Color,
    ) -> Result<&mut Self, AscendingError> {
        if points.len() < 2 {
            return Err(AscendingError::Other(OtherError::new(
                "MeshBuilder::polyline() got a list of < 2 points",
            )));
        }

        self.polyline_inner(mode, points, false, z, color)
    }

    pub fn polygon(
        &mut self,
        mode: DrawMode,
        points: &[Vec2],
        z: f32,
        color: Color,
    ) -> Result<&mut Self, AscendingError> {
        if points.len() < 3 {
            return Err(AscendingError::Other(OtherError::new(
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
    ) -> Result<&mut Self, AscendingError> {
        let vb = VertexBuilder {
            z,
            color,
            camera: self.use_camera,
        };
        self.polyline_with_vertex_builder(mode, points, is_closed, vb)
    }

    pub fn polyline_with_vertex_builder<V>(
        &mut self,
        mode: DrawMode,
        points: &[Vec2],
        is_closed: bool,
        vb: V,
    ) -> Result<&mut Self, AscendingError>
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
                .map(|p| {
                    let mint_point: mint::Point2<f32> = p.into();
                    tess::math::point(mint_point.x, mint_point.y)
                })
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

    pub fn rectangle(
        &mut self,
        mode: DrawMode,
        bounds: Vec4,
        z: f32,
        color: Color,
    ) -> Result<&mut Self, AscendingError> {
        {
            let buffers = &mut self.buffer;
            let rect = tess::math::Box2D::from_origin_and_size(
                tess::math::point(bounds.x, bounds.y),
                tess::math::size(bounds.z, bounds.w),
            );
            let vb = VertexBuilder {
                z,
                color,
                camera: self.use_camera,
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

    pub fn rounded_rectangle(
        &mut self,
        mode: DrawMode,
        bounds: Vec4,
        z: f32,
        radius: f32,
        color: Color,
    ) -> Result<&mut Self, AscendingError> {
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
                camera: self.use_camera,
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

    pub fn triangles(
        &mut self,
        triangles: &[Vec2],
        z: f32,
        color: Color,
    ) -> Result<&mut Self, AscendingError> {
        {
            if (triangles.len() % 3) != 0 {
                return Err(AscendingError::Other(OtherError::new(
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
                camera: self.use_camera,
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
