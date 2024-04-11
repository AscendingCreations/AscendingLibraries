mod pipeline;
mod render;
mod uniforms;
mod vertex;

pub use pipeline::*;
pub use render::*;
pub use uniforms::*;
pub use vertex::*;

use crate::{
    Color, DrawOrder, DrawType, GpuRenderer, Index, OrderedIndex, Vec2, Vec3,
    Vec4,
};
use slab::Slab;
use std::mem;
use wgpu::util::align_to;

pub const MAX_AREA_LIGHTS: usize = 2_000;
pub const MAX_DIR_LIGHTS: usize = 1_365;

pub struct AreaLight {
    pub pos: Vec2,
    pub color: Color,
    pub max_distance: f32,
    pub anim_speed: f32,
    pub dither: f32,
    pub animate: bool,
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
        }
    }
}

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
        }
    }
}

/// rendering data for world Light and all Lights.
pub struct Lights {
    pub world_color: Vec4,
    pub enable_lights: bool,
    pub store_id: Index,
    pub order: DrawOrder,
    pub render_layer: u32,
    pub area_lights: Slab<AreaLight>,
    pub directional_lights: Slab<DirectionalLight>,
    pub area_count: u32,
    pub dir_count: u32,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
    pub directionals_changed: bool,
    pub areas_changed: bool,
}

impl Lights {
    pub fn new(renderer: &mut GpuRenderer, render_layer: u32) -> Self {
        Self {
            world_color: Vec4::new(1.0, 1.0, 1.0, 0.0),
            enable_lights: false,
            store_id: renderer.new_buffer(),
            order: DrawOrder::default(),
            render_layer,
            area_lights: Slab::with_capacity(MAX_AREA_LIGHTS),
            directional_lights: Slab::with_capacity(MAX_DIR_LIGHTS),
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

    pub fn create_quad(&mut self, renderer: &mut GpuRenderer) {
        let instance = LightsVertex {
            world_color: self.world_color.to_array(),
            enable_lights: u32::from(self.enable_lights),
            dir_count: self.directional_lights.len() as u32,
            area_count: self.area_lights.len() as u32,
        };

        if let Some(store) = renderer.get_buffer_mut(self.store_id) {
            store.store = bytemuck::bytes_of(&instance).to_vec();
            store.changed = true;
        }
        let size = renderer.size();
        self.order = DrawOrder::new(
            false,
            &Vec3::default(),
            self.render_layer,
            &Vec2::new(size.width, size.height),
            DrawType::Lights,
        );
        self.changed = false;
    }

    pub fn insert_area_light(&mut self, light: AreaLight) -> Option<usize> {
        if self.area_lights.len() + 1 >= MAX_AREA_LIGHTS {
            return None;
        }

        self.areas_changed = true;
        self.changed = true;
        Some(self.area_lights.insert(light))
    }

    pub fn remove_area_light(&mut self, key: usize) {
        self.areas_changed = true;
        self.changed = true;
        self.area_lights.remove(key);
    }

    pub fn get_mut_area_light(&mut self, key: usize) -> Option<&mut AreaLight> {
        self.areas_changed = true;
        self.area_lights.get_mut(key)
    }

    pub fn insert_directional_light(
        &mut self,
        light: DirectionalLight,
    ) -> Option<usize> {
        if self.directional_lights.len() + 1 >= MAX_DIR_LIGHTS {
            return None;
        }

        self.directionals_changed = true;
        self.changed = true;
        Some(self.directional_lights.insert(light))
    }

    pub fn remove_directional_light(&mut self, key: usize) {
        self.directionals_changed = true;
        self.changed = true;
        self.directional_lights.remove(key);
    }

    pub fn get_mut_directional_light(
        &mut self,
        key: usize,
    ) -> Option<&mut DirectionalLight> {
        self.directionals_changed = true;
        self.directional_lights.get_mut(key)
    }

    /// used to check and update the vertex array.
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
