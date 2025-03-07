mod pipeline;
mod render;
mod uniforms;
mod vertex;

pub use pipeline::*;
pub use render::*;
pub use uniforms::*;
pub use vertex::*;

use crate::{
    CameraType, Color, DrawOrder, GpuRenderer, Index, OrderedIndex, Vec2, Vec3,
    Vec4,
};
use slotmap::SlotMap;
use std::mem;
use wgpu::util::align_to;

pub const MAX_AREA_LIGHTS: usize = 2_000;
pub const MAX_DIR_LIGHTS: usize = 1_333;

/// Area Lights rendered in the light system.
///
pub struct AreaLight {
    pub pos: Vec2,
    pub color: Color,
    pub max_distance: f32,
    pub anim_speed: f32,
    pub dither: f32,
    pub animate: bool,
    pub camera_type: CameraType,
}

impl AreaLight {
    fn to_raw(&self) -> AreaLightRaw {
        AreaLightRaw {
            pos: self.pos.to_array(),
            color: self.color.0,
            max_distance: self.max_distance,
            dither: self.dither,
            anim_speed: self.anim_speed,
            animate: u32::from(self.animate),
            camera_type: self.camera_type as u32,
        }
    }
}

/// Directional Lights rendered in the light system.
///
pub struct DirectionalLight {
    pub pos: Vec2,
    pub color: Color,
    pub max_distance: f32,
    pub max_width: f32,
    pub anim_speed: f32,
    pub angle: f32,
    pub dither: f32,
    pub fade_distance: f32,
    pub edge_fade_distance: f32,
    pub animate: bool,
    pub camera_type: CameraType,
}

impl DirectionalLight {
    fn to_raw(&self) -> DirectionalLightRaw {
        DirectionalLightRaw {
            pos: self.pos.to_array(),
            color: self.color.0,
            max_distance: self.max_distance,
            animate: u32::from(self.animate),
            max_width: self.max_width,
            anim_speed: self.anim_speed,
            dither: self.dither,
            angle: self.angle,
            fade_distance: self.fade_distance,
            edge_fade_distance: self.edge_fade_distance,
            camera_type: self.camera_type as u32,
        }
    }
}

/// Rendering data for world Light and all Lights.
pub struct Lights {
    /// Z Position of the Main Light Layer.
    pub z: f32,
    /// Color of the main light layer.
    pub world_color: Vec4,
    /// If the [`AreaLight`] and [`DirectionalLight`] are enabled.
    pub enable_lights: bool,
    /// [`Index`] of the Rendering Buffer.
    pub store_id: Index,
    /// DrawOrder of the world Lights.
    pub order: DrawOrder,
    /// SlotMap storage of [`AreaLight`]'s.
    pub area_lights: SlotMap<Index, AreaLight>,
    /// SlotMap storage of [`DirectionalLight`]'s.
    pub directional_lights: SlotMap<Index, DirectionalLight>,
    /// Count of [`AreaLight`]'s.
    pub area_count: u32,
    /// Count of [`DirectionalLight`]'s.
    pub dir_count: u32,
    /// If Main Light layer got updated we need to update the buffers too.
    pub changed: bool,
    /// If any [`DirectionalLight`] got updated we need to update the buffers too.
    pub directionals_changed: bool,
    /// If any [`AreaLight`] got updated we need to update the buffers too.
    pub areas_changed: bool,
}

impl Lights {
    /// Creates a new [`Lights`].
    ///
    /// order_layer: Rendering Layer of the world lights used in DrawOrder.
    pub fn new(renderer: &mut GpuRenderer, order_layer: u32, z: f32) -> Self {
        Self {
            z,
            world_color: Vec4::new(1.0, 1.0, 1.0, 0.0),
            enable_lights: false,
            store_id: renderer.new_buffer(
                bytemuck::bytes_of(&LightsVertex::default()).len(),
                0,
            ),
            order: DrawOrder::new(
                true,
                Vec3::new(0.0, 0.0, z),
                order_layer,
            ),
            area_lights: SlotMap::with_capacity_and_key(MAX_AREA_LIGHTS),
            directional_lights: SlotMap::with_capacity_and_key(MAX_DIR_LIGHTS),
            area_count: 0,
            dir_count: 0,
            changed: true,
            directionals_changed: true,
            areas_changed: true,
        }
    }

    pub fn unload(&self, renderer: &mut GpuRenderer) {
        renderer.remove_buffer(self.store_id);
    }

    pub fn set_world_color(&mut self, color: Vec4) -> &mut Self {
        self.world_color = color;
        self.order.alpha = color.w < 1.0;
        self.changed = true;
        self
    }

    /// Updates the [`Lights`]'s [`DrawOrder`]'s is Alpha.
    /// Use this after set_color to overide the alpha sorting.
    ///
    pub fn set_order_alpha(&mut self, alpha: bool) -> &mut Self {
        self.order.alpha = alpha;
        self
    }

    pub fn set_z_pos(&mut self, z: f32) -> &mut Self {
        self.z = z;
        self.order.set_position(Vec3::new(0.0, 0.0, z));
        self.changed = true;
        self
    }

    /// Updates the [`Lights`]'s Buffers to prepare them for rendering.
    ///
    pub fn create_quad(&mut self, renderer: &mut GpuRenderer) {
        let instance = LightsVertex {
            world_color: self.world_color.to_array(),
            enable_lights: u32::from(self.enable_lights),
            dir_count: self.directional_lights.len() as u32,
            area_count: self.area_lights.len() as u32,
            z: self.z,
        };

        if let Some(store) = renderer.get_buffer_mut(self.store_id) {
            let bytes = bytemuck::bytes_of(&instance);

            if bytes.len() != store.store.len() {
                store.store.resize_with(bytes.len(), || 0);
            }

            store.store.copy_from_slice(bytes);
            store.changed = true;
        }

        self.changed = false;
    }

    /// Inserts a [`AreaLight`] into [`Lights`].
    /// Returns the [`AreaLight`]'s [`Index`].
    ///
    pub fn insert_area_light(&mut self, light: AreaLight) -> Option<Index> {
        if self.area_lights.len() + 1 >= MAX_AREA_LIGHTS {
            return None;
        }

        self.areas_changed = true;
        self.changed = true;
        Some(self.area_lights.insert(light))
    }

    /// Removes a [`AreaLight`] by its [`Index`].
    ///
    pub fn remove_area_light(&mut self, key: Index) {
        self.areas_changed = true;
        self.changed = true;
        self.area_lights.remove(key);
    }

    /// Gets a Optional mutable reference of a [`Index`]ed [`AreaLight`].
    ///
    pub fn get_mut_area_light(&mut self, key: Index) -> Option<&mut AreaLight> {
        self.areas_changed = true;
        self.area_lights.get_mut(key)
    }

    /// Inserts a [`DirectionalLight`] into [`Lights`].
    /// Returns the [`DirectionalLight`]'s [`Index`].
    ///
    pub fn insert_directional_light(
        &mut self,
        light: DirectionalLight,
    ) -> Option<Index> {
        if self.directional_lights.len() + 1 >= MAX_DIR_LIGHTS {
            return None;
        }

        self.directionals_changed = true;
        self.changed = true;
        Some(self.directional_lights.insert(light))
    }

    /// Removes a [`DirectionalLight`] by its [`Index`].
    ///
    pub fn remove_directional_light(&mut self, key: Index) {
        self.directionals_changed = true;
        self.changed = true;
        self.directional_lights.remove(key);
    }

    /// Gets a Optional mutable reference of a [`Index`]ed [`DirectionalLight`].
    ///
    pub fn get_mut_directional_light(
        &mut self,
        key: Index,
    ) -> Option<&mut DirectionalLight> {
        self.directionals_changed = true;
        self.directional_lights.get_mut(key)
    }

    /// Used to check and update the vertex array.
    /// Returns a [`OrderedIndex`] used in Rendering.
    ///
    pub fn update(
        &mut self,
        renderer: &mut GpuRenderer,
        areas: &mut wgpu::Buffer,
        dirs: &mut wgpu::Buffer,
    ) -> OrderedIndex {
        // if pos or tex_pos or color changed.
        if self.changed {
            self.create_quad(renderer);
        }

        if self.areas_changed {
            let area_alignment: usize =
                align_to(mem::size_of::<AreaLightRaw>(), 32) as usize;
            for (i, (_key, light)) in self.area_lights.iter().enumerate() {
                renderer.queue().write_buffer(
                    areas,
                    (i * area_alignment) as wgpu::BufferAddress,
                    bytemuck::bytes_of(&light.to_raw()),
                );
            }

            self.areas_changed = false;
        }

        if self.directionals_changed {
            let dir_alignment: usize =
                align_to(mem::size_of::<DirectionalLightRaw>(), 48) as usize;
            for (i, (_key, dir)) in self.directional_lights.iter().enumerate() {
                renderer.queue().write_buffer(
                    dirs,
                    (i * dir_alignment) as wgpu::BufferAddress,
                    bytemuck::bytes_of(&dir.to_raw()),
                );
            }

            self.directionals_changed = false;
        }

        OrderedIndex::new(self.order, self.store_id, 0)
    }
}
