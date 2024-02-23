use crate::{Allocation, AscendingError, AtlasSet, GpuRenderer, TileSheet};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::{
    io::{Error, ErrorKind},
    path::Path,
};

#[derive(Clone, Debug, Default)]
pub struct Texture {
    name: String,
    pub bytes: Vec<u8>,
    size: (u32, u32),
}

impl Texture {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, AscendingError> {
        let name = path
            .as_ref()
            .to_str()
            .ok_or_else(|| {
                Error::new(ErrorKind::Other, "could not convert name to String")
            })?
            .to_owned();

        Ok(Self::from_image(name, image::open(path)?))
    }

    pub fn upload_from(
        path: impl AsRef<Path>,
        atlas: &mut AtlasSet<String, i32>,
        renderer: &GpuRenderer,
    ) -> Option<usize> {
        let name = path.as_ref().to_str()?.to_owned();

        if let Some(id) = atlas.lookup(&name) {
            Some(id)
        } else {
            let texture = Texture::from_file(path).ok()?;
            let (width, height) = texture.size();
            atlas.upload(name, texture.bytes(), width, height, 0, renderer)
        }
    }

    pub fn upload_from_with_alloc(
        path: impl AsRef<Path>,
        atlas: &mut AtlasSet<String, i32>,
        renderer: &GpuRenderer,
    ) -> Option<(usize, Allocation)> {
        let name = path.as_ref().to_str()?.to_owned();

        if let Some(id) = atlas.lookup(&name) {
            atlas.peek(id).map(|(allocation, _)| (id, *allocation))
        } else {
            let texture = Texture::from_file(path).ok()?;
            let (width, height) = texture.size();
            atlas.upload_with_alloc(
                name,
                texture.bytes(),
                width,
                height,
                0,
                renderer,
            )
        }
    }

    pub fn from_image(name: String, image: DynamicImage) -> Self {
        let size = image.dimensions();
        let bytes = image.into_rgba8().into_raw();

        Self { name, bytes, size }
    }

    pub fn from_memory(
        name: String,
        data: &[u8],
    ) -> Result<Self, AscendingError> {
        Ok(Self::from_image(name, image::load_from_memory(data)?))
    }

    pub fn from_memory_with_format(
        name: String,
        data: &[u8],
        format: ImageFormat,
    ) -> Result<Self, AscendingError> {
        Ok(Self::from_image(
            name,
            image::load_from_memory_with_format(data, format)?,
        ))
    }

    pub fn upload(
        &self,
        atlas: &mut AtlasSet<String, i32>,
        renderer: &GpuRenderer,
    ) -> Option<usize> {
        let (width, height) = self.size;
        atlas.upload(self.name.clone(), &self.bytes, width, height, 0, renderer)
    }

    pub fn upload_with_alloc(
        &self,
        atlas: &mut AtlasSet<String, i32>,
        renderer: &GpuRenderer,
    ) -> Option<(usize, Allocation)> {
        let (width, height) = self.size;
        atlas.upload_with_alloc(
            self.name.clone(),
            &self.bytes,
            width,
            height,
            0,
            renderer,
        )
    }

    pub fn new_tilesheet(
        self,
        atlas: &mut AtlasSet<String, i32>,
        renderer: &GpuRenderer,
        tilesize: u32,
    ) -> Option<TileSheet> {
        TileSheet::new(self, renderer, atlas, tilesize)
    }

    pub fn tilesheet_upload(
        self,
        tilesheet: &mut TileSheet,
        atlas: &mut AtlasSet<String, i32>,
        renderer: &GpuRenderer,
        tilesize: u32,
    ) -> Option<()> {
        tilesheet.upload(self, renderer, atlas, tilesize)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}
