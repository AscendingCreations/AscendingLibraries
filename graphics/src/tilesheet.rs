use crate::{AtlasSet, GpuRenderer, Texture};
use image::{EncodableLayout, ImageBuffer, RgbaImage};

/// Used to map the tile in the tilesheet back visually
/// this is only needed for the Editor.
///
#[derive(Debug)]
pub struct Tile {
    /// Location of the tile within the loaded Texture.
    pub x: u32,
    pub y: u32,
    /// Texture ID to reload the above if needed.
    pub tex_id: usize,
}

#[derive(Debug, Default)]
/// Loads the tiles from a tilesheet into a texture.
/// If this is used then you can not unload tiles or add new tiles
/// to any tilesheet loaded in th emiddle other than the very last tilesheet.
///
pub struct TileSheet {
    pub tiles: Vec<Tile>,
}

impl TileSheet {
    /// Creates a New [`TileSheet`] from a [`Texture`].
    /// This splits the [`Texture`] into [`Tile`]'s using tilesize and places them into the [`AtlasSet`].
    ///
    /// Returns a Optional [`TileSheet`] upon successful completion.
    ///
    pub fn new(
        texture: Texture,
        renderer: &GpuRenderer,
        atlas: &mut AtlasSet<String, i32>,
        tilesize: u32,
    ) -> Option<TileSheet> {
        let tilecount =
            (texture.size().0 / tilesize) * (texture.size().1 / tilesize);
        let sheet_width = texture.size().0 / tilesize;
        let sheet_image: RgbaImage = ImageBuffer::from_raw(
            texture.size().0,
            texture.size().1,
            texture.bytes.to_owned(),
        )
        .unwrap_or(ImageBuffer::new(texture.size().0, texture.size().1));
        let mut tiles = Vec::with_capacity(tilecount as usize);

        // lets check this to add in the empty tile set first if nothing else yet exists.
        // Also lets add the black tile.
        let empty = if let Some(empty) = atlas.lookup(&"Empty".to_owned()) {
            empty
        } else {
            let image: RgbaImage = ImageBuffer::new(tilesize, tilesize);
            atlas.upload(
                "Empty".to_owned(),
                image.as_bytes(),
                tilesize,
                tilesize,
                0,
                renderer,
            )?
        };

        for id in 0..tilecount {
            let mut image: RgbaImage = ImageBuffer::new(tilesize, tilesize);
            // get its location to remap it back visually.
            let (tilex, tiley) = (
                ((id % sheet_width) * tilesize),
                ((id / sheet_width) * tilesize),
            );

            // lets create the tile from the texture.
            for y in 0..tilesize {
                for x in 0..tilesize {
                    let pixel = sheet_image.get_pixel(tilex + x, tiley + y);
                    image.put_pixel(x, y, *pixel);
                }
            }

            if image.enumerate_pixels().all(|p| p.2.0[3] == 0) {
                // lets use our only Blank tile. this will always be the first loaded.
                // We use this when tiles are empty to avoid issues later when we do use
                // these spots for other tiles.
                tiles.push(Tile {
                    x: tilex,
                    y: tiley,
                    tex_id: empty,
                })
            } else {
                let name: String = format!("{}-{}", texture.name(), id);
                let tex_id = atlas.upload(
                    name,
                    image.as_bytes(),
                    tilesize,
                    tilesize,
                    0,
                    renderer,
                )?;

                tiles.push(Tile {
                    x: tilex,
                    y: tiley,
                    tex_id,
                })
            }
        }

        // We return as Some(tilesheet) this allows us to check above upon
        // upload if a tile failed to get added or not due to no more room.
        Some(TileSheet { tiles })
    }

    /// Appends new [`Tile`] from a [`Texture`].
    /// This splits the [`Texture`] into [`Tile`]'s using tilesize and places them into the [`AtlasSet`].
    ///
    /// Returns a Some(()) upon successful completion.
    ///
    pub fn upload(
        &mut self,
        texture: Texture,
        renderer: &GpuRenderer,
        atlas: &mut AtlasSet<String, i32>,
        tilesize: u32,
    ) -> Option<()> {
        let tilecount =
            (texture.size().0 / tilesize) * (texture.size().1 / tilesize);
        let sheet_width = texture.size().0 / tilesize;
        let sheet_image: RgbaImage = ImageBuffer::from_raw(
            texture.size().0,
            texture.size().1,
            texture.bytes.to_owned(),
        )
        .unwrap_or(ImageBuffer::new(texture.size().0, texture.size().1));

        // lets check this to add in the empty tile set first if nothing else yet exists.
        // Also lets add the black tile.
        let empty = if let Some(empty) = atlas.lookup(&"Empty".to_owned()) {
            empty
        } else {
            let image: RgbaImage = ImageBuffer::new(tilesize, tilesize);
            atlas.upload(
                "Empty".to_owned(),
                image.as_bytes(),
                tilesize,
                tilesize,
                0,
                renderer,
            )?
        };

        for id in 0..tilecount {
            let (tilex, tiley) = (
                ((id % sheet_width) * tilesize),
                ((id / sheet_width) * tilesize),
            );
            let mut image: RgbaImage = ImageBuffer::new(tilesize, tilesize);

            // lets create the tile from the texture.
            for y in 0..tilesize {
                for x in 0..tilesize {
                    let pixel = sheet_image.get_pixel(tilex + x, tiley + y);
                    image.put_pixel(x, y, *pixel);
                }
            }

            if image.enumerate_pixels().all(|p| p.2.0[3] == 0) {
                self.tiles.push(Tile {
                    x: tilex,
                    y: tiley,
                    tex_id: empty,
                })
            } else {
                let name: String = format!("{}-{}", texture.name(), id);
                let tex_id = atlas.upload(
                    name,
                    image.as_bytes(),
                    tilesize,
                    tilesize,
                    0,
                    renderer,
                )?;

                self.tiles.push(Tile {
                    x: tilex,
                    y: tiley,
                    tex_id,
                })
            }
        }

        Some(())
    }
}
