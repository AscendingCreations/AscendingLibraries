use crate::{FxHashMap, GpuDevice};
use bytemuck::{Pod, Zeroable};
use std::{
    any::{Any, TypeId},
    rc::Rc,
};

pub trait Layout: Pod + Zeroable {
    fn create_layout(
        &self,
        gpu_device: &mut GpuDevice,
    ) -> wgpu::BindGroupLayout;

    fn layout_key(&self) -> (TypeId, Vec<u8>) {
        let type_id = self.type_id();
        let bytes: Vec<u8> =
            bytemuck::try_cast_slice(&[*self]).unwrap_or(&[]).to_vec();

        (type_id, bytes)
    }
}

pub struct LayoutStorage {
    pub(crate) bind_group_map:
        FxHashMap<(TypeId, Vec<u8>), Rc<wgpu::BindGroupLayout>>,
}

impl LayoutStorage {
    pub fn new() -> Self {
        Self {
            bind_group_map: FxHashMap::default(),
        }
    }

    pub fn create_layout<K: Layout>(
        &mut self,
        device: &mut GpuDevice,
        layout: K,
    ) -> Rc<wgpu::BindGroupLayout> {
        let key = layout.layout_key();

        let layout = self
            .bind_group_map
            .entry(key)
            .or_insert_with(|| Rc::new(layout.create_layout(device)));

        Rc::clone(layout)
    }
}

impl Default for LayoutStorage {
    fn default() -> Self {
        Self::new()
    }
}
