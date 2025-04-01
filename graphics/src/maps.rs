mod pipeline;
mod render;
mod vertex;

pub use pipeline::*;
pub use render::*;
pub use vertex::*;

use std::iter;

use crate::{
    AtlasSet, CameraType, DrawOrder, GpuRenderer, Index, OrderedIndex, Vec2,
    Vec3,
};
use cosmic_text::Color;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum MapLayers {
    Ground,
    Mask,
    /// Mask 2 is the Z layer spacer for bridges.
    Mask2,
    Anim1,
    Anim2,
    Anim3,
    Anim4,
    /// always above player. \/
    Fringe,
    Fringe2,
    Count,
}

impl MapLayers {
    pub const LAYERS: [Self; 9] = [
        Self::Ground,
        Self::Mask,
        Self::Mask2,
        Self::Anim1,
        Self::Anim2,
        Self::Anim3,
        Self::Anim4,
        Self::Fringe,
        Self::Fringe2,
    ];

    pub fn indexed_layers(self) -> f32 {
        match self {
            Self::Ground => 9.6,
            Self::Mask => 9.5,
            Self::Mask2 => 9.4,
            Self::Anim1 => 9.3,
            Self::Anim2 => 9.2,
            Self::Anim3 => 9.1,
            Self::Anim4 => 9.0,
            Self::Fringe => 5.1,
            _ => 5.0,
        }
    }

    pub fn as_str<'a>(self) -> &'a str {
        match self {
            Self::Ground => "Ground",
            Self::Mask => "Mask",
            Self::Mask2 => "Mask 2",
            Self::Anim1 => "Anim 1",
            Self::Anim2 => "Anim 2",
            Self::Anim3 => "Anim 3",
            Self::Anim4 => "Anim 4",
            Self::Fringe => "Fringe",
            _ => "Fringe 2",
        }
    }
}

#[derive(Copy, Clone)]
pub struct TileData {
    ///tiles allocation ID within the texture.
    pub id: usize,
    pub color: Color,
}

impl Default for TileData {
    fn default() -> Self {
        Self {
            id: 0,
            color: Color::rgba(255, 255, 255, 255),
        }
    }
}

pub const TILE_COUNT: usize = 9216;
pub const LOWER_COUNT: usize = 7168;
pub const UPPER_COUNT: usize = 2048;

/// Handler for rendering Map to GPU.
pub struct Map {
    /// X, Y, GroupID for loaded map.
    /// Add this to the higher up Map struct.
    /// pub world_pos: Vec3,
    /// its render position. within the screen.
    pub pos: Vec2,
    // tiles per layer.
    pub tiles: Vec<TileData>,
    pub lower_buffer: Vec<MapVertex>,
    pub upper_buffer: Vec<MapVertex>,
    /// Store index per each layer.
    pub stores: [Index; 2],
    /// the draw order of the maps. created when update is called.
    pub orders: [DrawOrder; 2],
    /// count if any Filled Tiles Exist. this is to optimize out empty maps in rendering.
    pub filled_tiles: [u16; MapLayers::Count as usize],
    /// The size of the Tile to render. for spacing tiles out upon
    /// vertex creation. Default will be 20.
    pub tilesize: u32,
    /// Used to deturmine if the map can be rendered or if its just a preload.
    pub can_render: bool,
    pub camera_type: CameraType,
    /// If the position or a tile gets changed.
    pub changed: bool,
}

impl Map {
    /// Updates the [`Map`]'s Buffers to prepare them for rendering.
    ///
    pub fn create_quad(
        &mut self,
        renderer: &mut GpuRenderer,
        atlas: &mut AtlasSet,
    ) {
        let atlas_width = atlas.size().x / self.tilesize;

        self.lower_buffer.clear();
        self.upper_buffer.clear();

        for layer in MapLayers::LAYERS {
            let z = layer.indexed_layers();

            if self.filled_tiles[layer as usize] == 0 {
                continue;
            }

            for x in 0..32 {
                for y in 0..32 {
                    let tile = &self.tiles
                        [(x + (y * 32) + (layer as u32 * 1024)) as usize];

                    if tile.id == 0 {
                        continue;
                    }

                    if let Some((allocation, _)) = atlas.peek(tile.id) {
                        let (posx, posy) = allocation.position();

                        let map_vertex = MapVertex {
                            position: [
                                self.pos.x + (x * self.tilesize) as f32,
                                self.pos.y + (y * self.tilesize) as f32,
                                z,
                            ],
                            tilesize: self.tilesize as f32,
                            tile_id: (posx / self.tilesize)
                                + ((posy / self.tilesize) * atlas_width),
                            texture_layer: allocation.layer as u32,
                            color: tile.color.0,
                            camera_type: self.camera_type as u32,
                        };

                        if layer < MapLayers::Fringe {
                            self.lower_buffer.push(map_vertex)
                        } else {
                            self.upper_buffer.push(map_vertex)
                        }
                    }
                }
            }
        }

        if let Some(store) = renderer.get_buffer_mut(self.stores[0]) {
            let bytes = bytemuck::cast_slice(&self.lower_buffer);

            if bytes.len() != store.store.len() {
                store.store.resize_with(bytes.len(), || 0);
            }

            store.store.copy_from_slice(bytes);
            store.changed = true;
        }

        if let Some(store) = renderer.get_buffer_mut(self.stores[1]) {
            let bytes = bytemuck::cast_slice(&self.upper_buffer);

            if bytes.len() != store.store.len() {
                store.store.resize_with(bytes.len(), || 0);
            }

            store.store.copy_from_slice(bytes);
            store.changed = true;
        }
    }

    /// Creates a new [`Map`] with tilesize.
    ///
    pub fn new(renderer: &mut GpuRenderer, tilesize: u32) -> Self {
        let map_vertex_size = bytemuck::bytes_of(&MapVertex::default()).len();

        let lower_index = renderer.new_buffer(map_vertex_size * LOWER_COUNT, 0);
        let upper_index = renderer.new_buffer(map_vertex_size * UPPER_COUNT, 0);

        let order1 = DrawOrder::new(false, Vec3::new(0.0, 0.0, 9.0), 0);

        let order2 = DrawOrder::new(false, Vec3::new(0.0, 0.0, 5.0), 1);

        Self {
            tiles: iter::repeat_n(TileData::default(), 9216).collect(),
            pos: Vec2::default(),
            stores: [lower_index, upper_index],
            filled_tiles: [0; MapLayers::Count as usize],
            lower_buffer: Vec::with_capacity(LOWER_COUNT),
            upper_buffer: Vec::with_capacity(UPPER_COUNT),
            orders: [order1, order2],
            tilesize,
            can_render: false,
            changed: true,
            camera_type: CameraType::None,
        }
    }

    /// Updates the [`Map`]'s position.
    ///
    pub fn set_position(&mut self, position: Vec2) -> &mut Self {
        self.orders[0].set_position(Vec3::new(position.x, position.y, 9.0));
        self.orders[1].set_position(Vec3::new(position.x, position.y, 5.0));
        self.pos = position;
        self.changed = true;
        self
    }

    /// Updates the [`Map`]'s orders to overide the last set position.
    /// Use this after calls to set_position to set it to a order.
    ///
    pub fn set_order_pos(&mut self, order_override: Vec2) -> &mut Self {
        self.orders[0].set_position(Vec3::new(
            order_override.x,
            order_override.y,
            9.0,
        ));
        self.orders[1].set_position(Vec3::new(
            order_override.x,
            order_override.y,
            5.0,
        ));

        self
    }

    /// Updates one of the [`Map`]'s order Layer.
    ///
    /// Default Orders for Layer 1 is 0 Layer 2 is 1.
    pub fn set_order_layer(
        &mut self,
        index: usize,
        order_layer: u32,
    ) -> &mut Self {
        if let Some(order) = self.orders.get_mut(index) {
            order.order_layer = order_layer;
        }

        self
    }

    /// Unloades the [`Map`]'s buffer from the buffer store.
    ///
    pub fn unload(&self, renderer: &mut GpuRenderer) {
        for index in &self.stores {
            renderer.remove_buffer(*index);
        }
    }

    /// gets the [`TileData`] based upon the tiles x, y, and [`MapLayers`].
    /// [`MapLayers::Ground`] is Layer 0.
    ///
    pub fn get_tile(&self, pos: (u32, u32, u32)) -> TileData {
        assert!(
            pos.0 < 32 || pos.1 < 32 || pos.2 < 9,
            "pos is invalid. X < 32, y < 256, z < 9"
        );

        self.tiles[(pos.0 + (pos.1 * 32) + (pos.2 * 1024)) as usize]
    }

    /// Sets the [`CameraType`] this object will use to Render with.
    ///
    pub fn set_camera_type(&mut self, camera_type: CameraType) -> &mut Self {
        self.camera_type = camera_type;
        self.changed = true;
        self
    }

    /// This sets the tile's Id within the texture,
    /// layer within the texture array and Alpha for its transparency.
    /// This allows us to loop through the tiles Shader side efficiently.
    ///
    pub fn set_tile(&mut self, pos: (u32, u32, u32), tile: TileData) {
        if pos.0 >= 32 || pos.1 >= 32 || pos.2 >= 9 {
            return;
        }
        let tilepos = (pos.0 + (pos.1 * 32) + (pos.2 * 1024)) as usize;
        let current_tile = self.tiles[tilepos];

        if (current_tile.id > 0 && current_tile.color.a() > 0)
            && (tile.color.a() == 0 || tile.id == 0)
        {
            self.filled_tiles[pos.2 as usize] =
                self.filled_tiles[pos.2 as usize].saturating_sub(1);
        } else if tile.color.a() > 0
            && tile.id > 0
            && (current_tile.id == 0 || current_tile.color.a() == 0)
        {
            self.filled_tiles[pos.2 as usize] =
                self.filled_tiles[pos.2 as usize].saturating_add(1);
        }

        self.tiles[tilepos] = tile;
        self.changed = true;
    }

    /// Used to check and update the [`Map`]'s Buffer for Rendering.
    /// Returns an Optional vec![Lower, Upper] [`OrderedIndex`] to use in Rendering.
    ///
    pub fn update(
        &mut self,
        renderer: &mut GpuRenderer,
        atlas: &mut AtlasSet,
    ) -> Option<Vec<OrderedIndex>> {
        if self.can_render {
            if self.changed {
                self.create_quad(renderer, atlas);
                self.changed = false;
            }

            let orders = (0..2)
                .map(|i| OrderedIndex::new(self.orders[i], self.stores[i], 0))
                .collect();
            Some(orders)
        } else {
            None
        }
    }
}
