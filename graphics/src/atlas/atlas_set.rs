use crate::{
    AHashMap, AHashSet, Allocation, Atlas, GpuRenderer, Index, TextureGroup,
    TextureLayout, UVec3,
};
use lru::LruCache;
use slotmap::SlotMap;
use std::hash::Hash;
use wgpu::BindGroup;
/**
 * AtlasSet is used to hold and contain the data of many Atlas layers.
 * Each Atlas keeps track of the allocations allowed. Each allocation is a
 * given Width/Height as well as Position that a Texture image can fit within
 * the atlas.
 *
 * We try to use Store to keep all Allocations localized so if they need to be
 * unloaded, migrated or replaced then the system can prevent improper rendering
 * using a outdated Allocation. We will also attempt to keep track of reference counts
 * loading the Index and try to keep track of LRU cache and a list of last used Indexs.
 * This will help reduce errors and can help to reduce Vram and Later Reduce Fragmentation
 * of the Atlas.
 *
 * *******************************FRAGMENTATION********************************************
 * Fragmentation of a Atlas is when you Deallocate and Allocate new image textures into the
 * Atlas. As this occurs there is a possibility that Small spots that can not be used in the
 * Atlas to appear. These small Sections might get merged into larger Sections upon Deallocation
 * of neighboring Allocations, But in some Cases these might over run the Atlas cuasing use to
 * use way more Vram than is needed. To fix this we must migrate all loaded Allocations to a new
 * Atlas and either move the old atlas to the back of the list for reuse or unload it. We can accomplish
 * knowing when to migrate the atlas by setting a deallocations_limit. We also can know when to unload a
 * empty layer by using the layer_free_limit. This will allow us to control VRam usage.
 *
 * TODO Keep track of Indexs within an Atlas.
 * TODO Create Migration Check function.
 * TODO Add way to tell if any texture needs to migrate.
 * TODO Add limitations to a migrating texture so we only move a bit at a time.
 * TODO Add Ability to Tell user through API that Vertexs and Indicies need to be
 * TODO reloaded upon migration changes.
 *
*/
pub struct AtlasSet<U: Hash + Eq + Clone = String, Data: Copy + Default = i32> {
    /// Texture in GRAM
    pub texture: wgpu::Texture,
    /// Texture View for WGPU
    pub texture_view: wgpu::TextureView,
    /// Layers of texture.
    pub layers: Vec<Atlas>,
    /// Holds the Original Texture Size and layer information.
    pub extent: wgpu::Extent3d,
    /// Store the Allocations se we can easily remove and update them.
    /// use a Generation id to avoid conflict if users use older allocation id's.
    /// Also stores the Key associated with the Allocation.
    pub store: SlotMap<Index, (Allocation<Data>, U)>,
    /// for key to index lookups.
    pub lookup: AHashMap<U, Index>,
    /// keeps a list of least used allocations so we can unload them when need be.
    /// Also include the RefCount per ID lookup.
    /// we use this to keep track of when Fonts need to be unloaded.
    /// this only helps to get memory back but does not fix fragmentation of the Atlas.
    pub cache: LruCache<Index, usize>,
    /// List of allocations used in the last frame to ensure we dont unload what is
    /// in use.
    pub last_used: AHashSet<Index>,
    /// Format the Texture uses.
    pub format: wgpu::TextureFormat,
    /// When the System will Error if reached. This is the max allowed Layers
    /// Default is 256 as Most GPU allow a max of 256.
    pub max_layers: usize,
    /// Limit of deallocations allowed before we attempt to migrate the textures
    /// allocations to fix fragmentation.
    /// TODO Think of better way to figure out fragmentations.
    pub deallocations_limit: usize,
    /// amount of layers in memory before we start checking for fragmentations.
    pub layer_check_limit: usize,
    /// When we should free empty layers. this must be more than 1 otherwise will cause
    /// issues.
    pub layer_free_limit: usize,
    /// uses the refcount to unload rather than the unused.
    /// must exist for fonts to unload correctly and must be set to false for them.
    pub use_ref_count: bool,
    /// Texture Bind group for Atlas
    pub texture_group: TextureGroup,
}

impl<U: Hash + Eq + Clone, Data: Copy + Default> AtlasSet<U, Data> {
    fn allocate(
        &mut self,
        width: u32,
        height: u32,
        data: Data,
    ) -> Option<Allocation<Data>> {
        /* Check if the allocation would fit. */
        if width > self.extent.width || height > self.extent.height {
            return None;
        }

        /* Try allocating from an existing layer. */
        for (i, layer) in self.layers.iter_mut().enumerate() {
            if let Some(allocation) = layer.allocator.allocate(width, height) {
                return Some(Allocation {
                    allocation,
                    layer: i,
                    data,
                });
            }
        }

        /* Try to see if we can clear out unused allocations first. */
        if !self.use_ref_count {
            loop {
                let (&id, _) = self.cache.peek_lru()?;

                //Check if ID has been used yet?
                if self.last_used.contains(&id) {
                    //Failed to find any unused allocations so lets try to add a layer.
                    break;
                }

                if let Some(layer_id) = self.remove(id) {
                    let layer = self.layers.get_mut(layer_id)?;

                    if let Some(allocation) =
                        layer.allocator.allocate(width, height)
                    {
                        return Some(Allocation {
                            allocation,
                            layer: layer_id,
                            data,
                        });
                    }
                }
            }
        }

        /* Add a new layer, as we found no layer to allocate from and could
        not retrieve any old allocations to use. */

        if self.layers.len() + 1 == self.max_layers {
            return None;
        }

        let mut layer = Atlas::new(self.extent.width);

        if let Some(allocation) = layer.allocator.allocate(width, height) {
            self.layers.push(layer);

            return Some(Allocation {
                allocation,
                layer: self.layers.len() - 1,
                data,
            });
        }

        /* We are out of luck. */
        None
    }

    //TODO Add shrink that takes layers using a unload boolean and also promote each layers
    //TODO allocation layers to the new layer location. while removing the old empty layer.
    fn grow(&mut self, amount: usize, renderer: &GpuRenderer) {
        if amount == 0 {
            return;
        }

        let extent = wgpu::Extent3d {
            width: self.extent.width,
            height: self.extent.height,
            depth_or_array_layers: self.layers.len() as u32,
        };

        let texture =
            renderer.device().create_texture(&wgpu::TextureDescriptor {
                label: Some("Texture"),
                size: extent,
                mip_level_count: 0,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[wgpu::TextureFormat::Bgra8Unorm],
            });

        let amount_to_copy = self.layers.len() - amount;

        let mut encoder = renderer.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Texture command encoder"),
            },
        );

        for (i, _) in self.layers.iter_mut().take(amount_to_copy).enumerate() {
            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTextureBase {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTextureBase {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: self.extent.width,
                    height: self.extent.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.texture = texture;
        self.texture_view =
            self.texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Texture Atlas"),
                format: Some(self.format),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(self.layers.len() as u32),
            });
        renderer.queue().submit(std::iter::once(encoder.finish()));
    }

    pub fn new(
        renderer: &mut GpuRenderer,
        format: wgpu::TextureFormat,
        use_ref_count: bool,
    ) -> Self {
        let limits = renderer.device().limits();
        let extent = wgpu::Extent3d {
            width: limits.max_texture_dimension_3d,
            height: limits.max_texture_dimension_3d,
            depth_or_array_layers: 2,
        };

        let texture =
            renderer.device().create_texture(&wgpu::TextureDescriptor {
                label: Some("Texture"),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[format],
            });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Texture Atlas"),
            format: Some(format),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
        });

        let texture_group =
            TextureGroup::from_view(renderer, &texture_view, TextureLayout);

        Self {
            texture,
            texture_view,
            layers: vec![
                Atlas::new(limits.max_texture_dimension_3d),
                Atlas::new(limits.max_texture_dimension_3d),
            ],
            store: SlotMap::with_capacity_and_key(512),
            lookup: AHashMap::new(),
            extent,
            cache: LruCache::unbounded(),
            last_used: AHashSet::default(),
            format,
            max_layers: limits.max_texture_array_layers as usize,
            deallocations_limit: 32,
            layer_check_limit: (limits.max_texture_array_layers as f64 * 0.8)
                as usize,
            layer_free_limit: 3,
            use_ref_count,
            texture_group,
        }
    }

    pub fn upload_allocation(
        &mut self,
        buffer: &[u8],
        allocation: &Allocation<Data>,
        renderer: &GpuRenderer,
    ) {
        let (x, y) = allocation.position();
        let (width, height) = allocation.size();
        let layer = allocation.layer;

        renderer.queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x,
                    y,
                    z: layer as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(
                    if self.format == wgpu::TextureFormat::Rgba8UnormSrgb {
                        4 * width
                    } else {
                        width
                    },
                ),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn clear(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.allocator.clear();
        }

        self.store.clear();
        self.lookup.clear();
        self.cache.clear();
        self.last_used.clear();
    }

    //TODO Make function that checks for unloading and migrating.
    pub fn trim(&mut self) {
        self.last_used.clear();
    }

    pub fn promote_by_key(&mut self, key: U) {
        if let Some(id) = self.lookup.get(&key) {
            self.cache.promote(id);
            self.last_used.insert(*id);
        }
    }

    pub fn promote(&mut self, id: Index) {
        self.cache.promote(&id);
        self.last_used.insert(id);
    }

    /// Get the ID of a image if it is loaded.
    pub fn lookup(&self, key: &U) -> Option<Index> {
        self.lookup.get(key).copied()
    }

    pub fn peek_by_key(&mut self, key: &U) -> Option<&(Allocation<Data>, U)> {
        if let Some(id) = self.lookup.get(key) {
            self.store.get(*id)
        } else {
            None
        }
    }

    pub fn peek(&mut self, id: Index) -> Option<&(Allocation<Data>, U)> {
        self.store.get(id)
    }

    pub fn contains_key(&mut self, key: &U) -> bool {
        self.lookup.contains_key(key)
    }

    pub fn contains(&mut self, id: Index) -> bool {
        self.store.contains_key(id)
    }

    pub fn get_by_key(&mut self, key: &U) -> Option<Allocation<Data>> {
        let id = *self.lookup.get(key)?;
        if let Some((allocation, _)) = self.store.get(id) {
            self.cache.promote(&id);
            self.last_used.insert(id);
            return Some(*allocation);
        }

        None
    }

    pub fn get(&mut self, id: Index) -> Option<Allocation<Data>> {
        if let Some((allocation, _)) = self.store.get(id) {
            self.cache.promote(&id);
            self.last_used.insert(id);
            return Some(*allocation);
        }

        None
    }

    /**
     * Removing will leave anything using the texture inable to load the correct texture if
     * a new texture is loaded in the olds place.
     * TODO Redo texture system so texture allocations are not held by the images but instead
     * TODO are held by the system so we can reload images later on if they got unloaded.
     *
     * returns the layer id if removed otherwise None for everything else.
     **/
    pub fn remove_by_key(&mut self, key: &U) -> Option<usize> {
        let id = *self.lookup.get(key)?;
        let refcount = self.cache.pop(&id)?.saturating_sub(1);

        if self.use_ref_count && refcount > 0 {
            self.cache.push(id, refcount);
            return None;
        }

        let (allocation, _) = self.store.remove(id)?;
        self.last_used.remove(&id);
        self.lookup.remove(key);
        self.layers
            .get_mut(allocation.layer)?
            .deallocate(id, allocation.allocation);
        Some(allocation.layer)
    }

    // returns the layer id if removed otherwise None for everything else.
    pub fn remove(&mut self, id: Index) -> Option<usize> {
        let refcount = self.cache.pop(&id)?.saturating_sub(1);

        if self.use_ref_count && refcount > 0 {
            self.cache.push(id, refcount);
            return None;
        }

        let (allocation, key) = self.store.remove(id)?;
        self.last_used.remove(&id);
        self.lookup.remove(&key);
        self.layers
            .get_mut(allocation.layer)?
            .deallocate(id, allocation.allocation);
        Some(allocation.layer)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upload(
        &mut self,
        key: U,
        bytes: &[u8],
        width: u32,
        height: u32,
        data: Data,
        renderer: &GpuRenderer,
    ) -> Option<Index> {
        if let Some(&id) = self.lookup.get(&key) {
            Some(id)
        } else {
            let allocation = {
                let nlayers = self.layers.len();
                let allocation = self.allocate(width, height, data)?;
                self.grow(self.layers.len() - nlayers, renderer);

                allocation
            };

            self.upload_allocation(bytes, &allocation, renderer);
            let id = self.store.insert((allocation, key.clone()));
            self.layers[allocation.layer].insert_index(id);
            self.lookup.insert(key, id);
            self.cache.push(id, 1);
            Some(id)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upload_with_alloc(
        &mut self,
        key: U,
        bytes: &[u8],
        width: u32,
        height: u32,
        data: Data,
        renderer: &GpuRenderer,
    ) -> Option<(Index, Allocation<Data>)> {
        if let Some(&id) = self.lookup.get(&key) {
            let (allocation, _) = self.store.get(id)?;
            Some((id, *allocation))
        } else {
            let allocation = {
                let nlayers = self.layers.len();
                let allocation = self.allocate(width, height, data)?;
                self.grow(self.layers.len() - nlayers, renderer);

                allocation
            };

            self.upload_allocation(bytes, &allocation, renderer);
            let id = self.store.insert((allocation, key.clone()));
            self.layers[allocation.layer].insert_index(id);
            self.lookup.insert(key.clone(), id);
            self.cache.push(id, 1);
            Some((id, allocation))
        }
    }

    pub fn size(&self) -> UVec3 {
        UVec3::new(
            self.extent.width,
            self.extent.height,
            self.extent.depth_or_array_layers,
        )
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.texture_group.bind_group
    }
}
