use crate::{
    Bounds, CameraType, Color, DrawOrder, GpuRenderer, GraphicsError, Index,
    OrderedIndex, TextAtlas, TextVertex, Vec2, Vec3,
};
use cosmic_text::{
    Attrs, Buffer, Cursor, FontSystem, Metrics, SwashCache, SwashContent, Wrap,
};

/// [`Text`] Option Handler for [`Text::measure_string`].
///
pub struct TextOptions {
    pub shaping: cosmic_text::Shaping,
    pub metrics: Option<Metrics>,
    pub buffer_width: Option<f32>,
    pub buffer_height: Option<f32>,
    pub scale: f32,
    pub wrap: Wrap,
}

/// [`Text`] visible width and lines details
pub struct VisibleDetails {
    /// Visible Width the Text can render as.
    pub width: f32,
    /// Visible Lines the Text can Render too.
    pub lines: usize,
    /// Max Height each line is rendered as.
    pub line_height: f32,
}

/// Text to render to screen.
///
pub struct Text {
    /// Cosmic Text [`Buffer`].
    pub buffer: Buffer,
    /// Position on the Screen.
    pub pos: Vec3,
    /// Width and Height of the Text Area.
    pub size: Vec2,
    /// Scale of the Text.
    pub scale: f32,
    /// Default Text Font Color.
    pub default_color: Color,
    /// Clip Bounds of Text.
    pub bounds: Bounds,
    /// Instance Buffer Store Index of Text Buffer.
    pub store_id: Index,
    /// Rendering Layer of the Text used in DrawOrder.
    pub render_layer: u32,
    /// the draw order of the Text. created/updated when update is called.
    pub order: DrawOrder,
    /// Cursor the shaping is set too.
    pub cursor: Cursor,
    /// line the shaping is set too.
    pub line: usize,
    /// set scroll to render too.
    pub scroll: cosmic_text::Scroll,
    /// Word Wrap Type. Default is Wrap::Word.
    pub wrap: Wrap,
    /// [`CameraType`] used to render with.
    pub camera_type: CameraType,
    /// Buffer used to Store Premade Glyphs.
    /// Avoids making new vec every create_quad call at risk of more memory.
    /// will only resize when resizing is needed
    pub glyph_vertices: Vec<TextVertex>,
    /// If anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Text {
    /// Updates the [`Text`]'s Buffers to prepare them for rendering.
    ///
    pub fn create_quad(
        &mut self,
        cache: &mut SwashCache,
        atlas: &mut TextAtlas,
        renderer: &mut GpuRenderer,
    ) -> Result<(), GraphicsError> {
        let count: usize =
            self.buffer.lines.iter().map(|line| line.text().len()).sum();

        self.glyph_vertices.clear();

        if self.glyph_vertices.capacity() < count {
            self.glyph_vertices.reserve_exact(count);
        }

        let mut is_alpha = false;
        let mut width = 0.0;
        let screensize = renderer.size();
        let bounds_min_x = self.bounds.left.max(0.0);
        let bounds_min_y = self.bounds.bottom.max(0.0);
        let bounds_max_x = self.bounds.right.min(screensize.width);
        let bounds_max_y = self.bounds.top.min(screensize.height);

        // From Glyphon good optimization.
        let is_run_visible = |run: &cosmic_text::LayoutRun| {
            let start_y = self.pos.y + self.size.y - run.line_top;
            let end_y = self.pos.y + self.size.y
                - run.line_top
                - (run.line_height * 0.5);

            start_y <= bounds_max_y + (run.line_height * 0.5)
                && bounds_min_y <= end_y
        };

        let layout_runs = self
            .buffer
            .layout_runs()
            .skip_while(|run| !is_run_visible(run))
            .take_while(is_run_visible);

        for run in layout_runs {
            width = run.line_w.max(width);

            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical(
                    (self.pos.x, self.pos.y + self.size.y),
                    self.scale,
                );

                let (allocation, is_color) = if let Some(allocation) =
                    atlas.text.get_by_key(&physical_glyph.cache_key)
                {
                    (allocation, false)
                } else if let Some(allocation) =
                    atlas.emoji.get_by_key(&physical_glyph.cache_key)
                {
                    (allocation, true)
                } else {
                    let image = cache
                        .get_image_uncached(
                            &mut renderer.font_sys,
                            physical_glyph.cache_key,
                        )
                        .unwrap();
                    let bitmap = image.data;
                    let is_color = match image.content {
                        SwashContent::Color => true,
                        SwashContent::Mask => false,
                        SwashContent::SubpixelMask => false,
                    };

                    let width = image.placement.width;
                    let height = image.placement.height;

                    if width > 0 && height > 0 {
                        if is_color {
                            let (_, allocation) = atlas
                                .emoji
                                .upload_with_alloc(
                                    physical_glyph.cache_key,
                                    &bitmap,
                                    width,
                                    height,
                                    Vec2::new(
                                        image.placement.left as f32,
                                        image.placement.top as f32,
                                    ),
                                    renderer,
                                )
                                .ok_or(GraphicsError::AtlasFull)?;
                            (allocation, is_color)
                        } else {
                            let (_, allocation) = atlas
                                .text
                                .upload_with_alloc(
                                    physical_glyph.cache_key,
                                    &bitmap,
                                    width,
                                    height,
                                    Vec2::new(
                                        image.placement.left as f32,
                                        image.placement.top as f32,
                                    ),
                                    renderer,
                                )
                                .ok_or(GraphicsError::AtlasFull)?;
                            (allocation, is_color)
                        }
                    } else {
                        continue;
                    }
                };

                let position = allocation.data;
                let (u, v, width, height) = allocation.rect();
                let (mut u, mut v, mut width, mut height) =
                    (u as f32, v as f32, width as f32, height as f32);

                let (mut x, mut y) = (
                    physical_glyph.x as f32 + position.x,
                    physical_glyph.y as f32
                        + ((position.y - height)
                            - (run.line_y * self.scale).round()),
                );

                let color = is_color
                    .then(|| Color::rgba(255, 255, 255, 255))
                    .unwrap_or(match glyph.color_opt {
                        Some(color) => color,
                        None => self.default_color,
                    });

                if color.a() < 255 {
                    is_alpha = true;
                }

                // Starts beyond right edge or ends beyond left edge
                let max_x = x + width;
                if x > bounds_max_x || max_x < bounds_min_x {
                    continue;
                }

                // Clip left edge
                if x < bounds_min_x {
                    let right_shift = bounds_min_x - x;

                    x = bounds_min_x;
                    width = max_x - bounds_min_x;
                    u += right_shift;
                }

                // Clip right edge
                if x + width > bounds_max_x {
                    width = bounds_max_x - x;
                }

                // Clip top edge
                if y < bounds_min_y {
                    height -= bounds_min_y - y;
                    y = bounds_min_y;
                }

                // Clip top edge
                if y + height > bounds_max_y {
                    let bottom_shift = (y + height) - bounds_max_y;

                    v += bottom_shift;
                    height -= bottom_shift;
                }

                let default = TextVertex {
                    position: [x, y, self.pos.z],
                    hw: [width, height],
                    tex_coord: [u, v],
                    layer: allocation.layer as u32,
                    color: color.0,
                    camera_type: self.camera_type as u32,
                    is_color: is_color as u32,
                };

                self.glyph_vertices.push(default);
            }
        }

        if let Some(store) = renderer.get_buffer_mut(self.store_id) {
            let bytes: &[u8] = bytemuck::cast_slice(&self.glyph_vertices);
            store.store.resize_with(bytes.len(), || 0);
            store.store.copy_from_slice(bytes);
            store.changed = true;
        }

        self.order = DrawOrder::new(is_alpha, &self.pos, self.render_layer);

        self.changed = false;
        self.buffer.set_redraw(false);
        Ok(())
    }

    /// Creates a new [`Text`].
    ///
    pub fn new(
        renderer: &mut GpuRenderer,
        metrics: Option<Metrics>,
        pos: Vec3,
        size: Vec2,
        scale: f32,
        render_layer: u32,
    ) -> Self {
        let text_starter_size =
            bytemuck::bytes_of(&TextVertex::default()).len() * 64;

        Self {
            buffer: Buffer::new(
                &mut renderer.font_sys,
                metrics.unwrap_or(Metrics::new(16.0, 16.0).scale(scale)),
            ),
            pos,
            size,
            bounds: Bounds::default(),
            store_id: renderer.new_buffer(text_starter_size, 0),
            order: DrawOrder::default(),
            changed: true,
            default_color: Color::rgba(0, 0, 0, 255),
            camera_type: CameraType::None,
            cursor: Cursor::default(),
            wrap: Wrap::Word,
            line: 0,
            scroll: cosmic_text::Scroll::default(),
            scale,
            render_layer,
            glyph_vertices: Vec::new(),
        }
    }

    /// Sets the [`Text`]'s [`CameraType`] for rendering.
    ///
    pub fn set_camera_type(&mut self, camera_type: CameraType) {
        self.camera_type = camera_type;
        self.changed = true;
    }

    /// Unloads the [`Text`] from the Instance Buffers Store.
    ///
    pub fn unload(&self, renderer: &mut GpuRenderer) {
        renderer.remove_buffer(self.store_id);
    }

    /// Resets the [`Text`] to contain the new text only.
    ///
    pub fn set_text(
        &mut self,
        renderer: &mut GpuRenderer,
        text: &str,
        attrs: Attrs,
        shaping: cosmic_text::Shaping,
    ) -> &mut Self {
        self.buffer
            .set_text(&mut renderer.font_sys, text, attrs, shaping);
        self.changed = true;
        self
    }

    /// Resets the [`Text`] to contain the new span of text only.
    ///
    pub fn set_rich_text<'r, 's, I>(
        &mut self,
        renderer: &mut GpuRenderer,
        spans: I,
        default_attr: Attrs,
        shaping: cosmic_text::Shaping,
    ) -> &mut Self
    where
        I: IntoIterator<Item = (&'s str, Attrs<'r>)>,
    {
        self.buffer.set_rich_text(
            &mut renderer.font_sys,
            spans,
            default_attr,
            shaping,
        );
        self.changed = true;
        self
    }

    /// For more advanced shaping and usage. Use [`Text::set_change`] to set if you need it to make changes or not.
    /// This will not set the change to true. when changes are made you must set changed to true.
    ///
    pub fn get_text_buffer(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// cursor shaping sets the [`Text`]'s location to shape from and sets the buffers scroll.
    ///
    pub fn shape_until_cursor(
        &mut self,
        renderer: &mut GpuRenderer,
        cursor: Cursor,
    ) -> &mut Self {
        if self.cursor != cursor || self.changed {
            self.cursor = cursor;
            self.line = 0;
            self.changed = true;
            self.buffer.shape_until_cursor(
                &mut renderer.font_sys,
                cursor,
                false,
            );
            self.scroll = self.buffer.scroll();
        }

        self
    }

    /// cursor shaping sets the [`Text`]'s location to shape from.
    ///
    pub fn shape_until(
        &mut self,
        renderer: &mut GpuRenderer,
        line: usize,
    ) -> &mut Self {
        if self.line != line || self.changed {
            self.cursor = Cursor::new(line, 0);
            self.line = line;
            self.changed = true;
            self.buffer.shape_until_cursor(
                &mut renderer.font_sys,
                self.cursor,
                false,
            );
        }
        self
    }

    /// scroll shaping sets the [`Text`]'s location to shape from.
    ///
    pub fn shape_until_scroll(
        &mut self,
        renderer: &mut GpuRenderer,
    ) -> &mut Self {
        if self.changed {
            self.buffer
                .shape_until_scroll(&mut renderer.font_sys, false);
        }

        self
    }

    /// sets scroll for shaping and sets the [`Text`]'s location to shape from.
    ///
    pub fn set_scroll(
        &mut self,
        renderer: &mut GpuRenderer,
        scroll: cosmic_text::Scroll,
    ) -> &mut Self {
        if self.scroll != scroll {
            self.scroll = scroll;
            self.buffer.set_scroll(scroll);
            self.changed = true;
            self.buffer
                .shape_until_scroll(&mut renderer.font_sys, false);
        }

        self
    }

    /// Sets the [`Text`] as changed for updating.
    ///
    pub fn set_change(&mut self, changed: bool) -> &mut Self {
        self.changed = changed;
        self
    }

    /// Sets the [`Text`] wrapping.
    ///
    pub fn set_wrap(
        &mut self,
        renderer: &mut GpuRenderer,
        wrap: Wrap,
    ) -> &mut Self {
        if self.wrap != wrap {
            self.wrap = wrap;
            self.buffer.set_wrap(&mut renderer.font_sys, wrap);
            self.changed = true;
        }

        self
    }

    /// Sets the [`Text`]'s clipping bounds.
    ///
    pub fn set_bounds(&mut self, bounds: Bounds) -> &mut Self {
        self.bounds = bounds;
        self.changed = true;
        self
    }

    /// Sets the [`Text`]'s screen Posaition.
    ///
    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.pos = position;
        self.changed = true;
        self
    }

    /// Sets the [`Text`]'s default color.
    ///
    pub fn set_default_color(&mut self, color: Color) -> &mut Self {
        self.default_color = color;
        self.changed = true;
        self
    }

    /// Sets the [`Text`]'s cosmic text buffer size.
    ///
    pub fn set_buffer_size(
        &mut self,
        renderer: &mut GpuRenderer,
        width: Option<f32>,
        height: Option<f32>,
    ) -> &mut Self {
        self.buffer.set_size(&mut renderer.font_sys, width, height);
        self.changed = true;
        self
    }

    /// clears the [`Text`] buffer.
    ///
    pub fn clear(&mut self, renderer: &mut GpuRenderer) -> &mut Self {
        self.buffer.set_text(
            &mut renderer.font_sys,
            "",
            cosmic_text::Attrs::new(),
            cosmic_text::Shaping::Basic,
        );
        self.changed = true;
        self
    }

    // Used to check and update the vertex array.
    /// Returns a [`OrderedIndex`] used in Rendering.
    ///
    pub fn update(
        &mut self,
        cache: &mut SwashCache,
        atlas: &mut TextAtlas,
        renderer: &mut GpuRenderer,
    ) -> Result<OrderedIndex, GraphicsError> {
        if self.changed {
            self.create_quad(cache, atlas, renderer)?;
        }

        Ok(OrderedIndex::new(self.order, self.store_id, 0))
    }

    /// Checks if mouse_pos is within the [`Text`]'s location.
    ///
    pub fn check_mouse_bounds(&self, mouse_pos: Vec2) -> bool {
        mouse_pos[0] > self.pos.x
            && mouse_pos[0] < self.pos.x + self.size.x
            && mouse_pos[1] > self.pos.y
            && mouse_pos[1] < self.pos.y + self.size.y
    }

    /// Returns Visible Width and Line details of the Rendered [`Text`].
    pub fn visible_details(&self) -> VisibleDetails {
        let (width, lines) = self.buffer.layout_runs().fold(
            (0.0, 0usize),
            |(width, total_lines), run| {
                (run.line_w.max(width), total_lines + 1)
            },
        );

        VisibleDetails {
            line_height: self.buffer.metrics().line_height,
            lines,
            width,
        }
    }

    /// measure's the [`Text`]'s Rendering Size.
    ///
    pub fn measure(&self) -> Vec2 {
        let details = self.visible_details();

        let (max_width, max_height) = self.buffer.size();
        let height = details.lines as f32 * details.line_height;

        Vec2::new(
            details
                .width
                .min(max_width.unwrap_or(0.0).max(details.width)),
            height.min(max_height.unwrap_or(0.0).max(height)),
        )
    }

    /// Allows measuring the String for how big it will be when Rendering.
    /// This will not create any buffers in the rendering system.
    ///
    pub fn measure_string(
        font_system: &mut FontSystem,
        text: &str,
        attrs: Attrs,
        options: TextOptions,
    ) -> Vec2 {
        let mut buffer = Buffer::new(
            font_system,
            options
                .metrics
                .unwrap_or(Metrics::new(16.0, 16.0).scale(options.scale)),
        );

        buffer.set_wrap(font_system, options.wrap);
        buffer.set_size(
            font_system,
            options.buffer_width,
            options.buffer_height,
        );
        buffer.set_text(font_system, text, attrs, options.shaping);

        let (width, total_lines) = buffer.layout_runs().fold(
            (0.0, 0usize),
            |(width, total_lines), run| {
                (run.line_w.max(width), total_lines + 1)
            },
        );

        let (max_width, max_height) = buffer.size();
        let height = total_lines as f32 * buffer.metrics().line_height;

        Vec2::new(
            width.min(max_width.unwrap_or(0.0).max(width)),
            height.min(max_height.unwrap_or(0.0).max(height)),
        )
    }
}
