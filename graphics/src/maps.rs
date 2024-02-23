mod pipeline;
mod render;
mod vertex;

pub use pipeline::*;
pub use render::*;
pub use vertex::*;

use std::iter;

use crate::{
    AtlasSet, DrawOrder, DrawType, GpuRenderer, Index, OrderedIndex, Vec2, Vec3,
};
use cosmic_text::Color;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum MapLayers {
    Ground,
    Mask,
    /// Mask 2 is the Z layer spacer for bridges.
    Mask2,
    Anim1,
    Anim2,
    Anim3,
    /// always above player. \/
    Fringe,
    Fringe2,
    Count,
}

impl MapLayers {
    pub const LAYERS: [Self; 8] = [
        Self::Ground,
        Self::Mask,
        Self::Mask2,
        Self::Anim1,
        Self::Anim2,
        Self::Anim3,
        Self::Fringe,
        Self::Fringe2,
    ];

    pub fn indexed_layers(layer: usize) -> f32 {
        match layer {
            0 => 9.5,
            1 => 9.4,
            2 => 9.3,
            3 => 9.2,
            4 => 9.1,
            5 => 9.0,
            6 => 5.1,
            _ => 5.0,
        }
    }

    pub fn as_str<'a>(layer: u32) -> &'a str {
        match layer {
            0 => "Ground",
            1 => "Mask",
            2 => "Mask 2",
            3 => "Anim 1",
            4 => "Anim 2",
            5 => "Anim 3",
            6 => "Fringe",
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

pub struct Map {
    /// X, Y, GroupID for loaded map.
    /// Add this to the higher up Map struct.
    /// pub world_pos: Vec3,
    /// its render position. within the screen.
    pub pos: Vec2,
    // tiles per layer.
    pub tiles: [TileData; 8192],
    /// Store index per each layer.
    pub stores: Vec<Index>,
    /// the draw order of the maps. created when update is called.
    pub orders: Vec<DrawOrder>,
    /// count if any Filled Tiles Exist. this is to optimize out empty maps in rendering.
    pub filled_tiles: [u16; MapLayers::Count as usize],
    // The size of the Tile to render. for spacing tiles out upon
    // vertex creation. Default will be 20.
    pub tilesize: u32,
    // Used to deturmine if the map can be rendered or if its just a preload.
    pub can_render: bool,
    /// if the position or a tile gets changed.
    pub changed: bool,
}

impl Map {
    pub fn create_quad(
        &mut self,
        renderer: &mut GpuRenderer,
        atlas: &mut AtlasSet,
    ) {
        let mut lower_buffer = Vec::with_capacity(6144);
        let mut upper_buffer = Vec::with_capacity(2048);
        let atlas_width = atlas.size().x / self.tilesize;

        for i in 0..8 {
            let z = MapLayers::indexed_layers(i);

            if self.filled_tiles[i] == 0 {
                continue;
            }

            for x in 0..32 {
                for y in 0..32 {
                    let tile = &self.tiles
                        [(x + (y * 32) + (i as u32 * 1024)) as usize];

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
                        };

                        if i < 6 {
                            lower_buffer.push(map_vertex)
                        } else {
                            upper_buffer.push(map_vertex)
                        }
                    }
                }
            }
        }

        let size = (self.tilesize * 32) as f32;

        if let Some(store) = renderer.get_buffer_mut(&self.stores[0]) {
            store.store = bytemuck::cast_slice(&lower_buffer).to_vec();
            store.changed = true;
        }

        if let Some(store) = renderer.get_buffer_mut(&self.stores[1]) {
            store.store = bytemuck::cast_slice(&upper_buffer).to_vec();
            store.changed = true;
        }

        self.orders[0] = DrawOrder::new(
            false,
            &Vec3::new(self.pos.x, self.pos.y, 9.0),
            0,
            &Vec2::new(size, size),
            DrawType::Map,
        );

        self.orders[1] = DrawOrder::new(
            false,
            &Vec3::new(self.pos.x, self.pos.y, 5.0),
            0,
            &Vec2::new(size, size),
            DrawType::Map,
        );
        self.changed = false;
    }

    pub fn new(renderer: &mut GpuRenderer, tilesize: u32) -> Self {
        Self {
            tiles: [TileData::default(); 8192],
            pos: Vec2::default(),
            stores: (0..2).map(|_| renderer.new_buffer()).collect(),
            filled_tiles: [0; MapLayers::Count as usize],
            orders: iter::repeat(DrawOrder::default()).take(2).collect(),
            tilesize,
            can_render: false,
            changed: true,
        }
    }

    pub fn get_tile(&self, pos: (u32, u32, u32)) -> TileData {
        assert!(
            pos.0 < 32 || pos.1 < 32 || pos.2 < 8,
            "pos is invalid. X < 32, y < 256, z < 8"
        );

        self.tiles[(pos.0 + (pos.1 * 32) + (pos.2 * 1024)) as usize]
    }

    // this sets the tile's Id within the texture,
    //layer within the texture array and Alpha for its transparency.
    // This allows us to loop through the tiles Shader side efficiently.
    pub fn set_tile(&mut self, pos: (u32, u32, u32), tile: TileData) {
        if pos.0 >= 32 || pos.1 >= 32 || pos.2 >= 8 {
            return;
        }
        let tilepos = (pos.0 + (pos.1 * 32) + (pos.2 * 1024)) as usize;
        let current_tile = self.tiles[tilepos];

        if (current_tile.id > 0 || current_tile.color.a() > 0)
            && (tile.color.a() == 0 || tile.id == 0)
        {
            self.filled_tiles[pos.2 as usize] =
                self.filled_tiles[pos.2 as usize].saturating_sub(1);
        } else if tile.color.a() > 0 || tile.id > 0 {
            self.filled_tiles[pos.2 as usize] =
                self.filled_tiles[pos.2 as usize].saturating_add(1);
        }

        self.tiles[tilepos] = tile;
        self.changed = true;
    }

    /// used to check and update the vertex array or Texture witht he image buffer.
    pub fn update(
        &mut self,
        renderer: &mut GpuRenderer,
        atlas: &mut AtlasSet,
    ) -> Option<Vec<OrderedIndex>> {
        if self.can_render {
            if self.changed {
                self.create_quad(renderer, atlas);
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
