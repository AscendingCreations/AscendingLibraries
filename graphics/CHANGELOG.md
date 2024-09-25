# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
## Unreleased

## 0.14.0 (25. September, 2024)
### Changed
- (Breaking) update_bounds for Rect returns &mut Self now.
- Updated Crates.

### Added
- Order Override to Image, Mesh2D, Text, Rect

## 0.13.0 (30. August, 2024)
### Changed
- Removed Text Offsets use Positions directly for this. Reduces memory a bit.
- (Breaking) Made Bounds not Optional. Text always looks at its bounds now. 
- Bounds now can determine if Text should even be iterated in the first place. (boosts Vertices and Glyph insertions speeds for invisible lines.)
- Added Glyph Buffer to reduce allocations. This will help improve performance but will increase Ram usage. 
- Removed Y Glyph discard check since new iterator should discard them.
- Optimized Shader Code.

## 0.12.1 (7. August, 2024)
### Fixed
- Map upper layer is now set to Render Layer 1. They can not be on the same layer as ground tiles due to Z rendering transparency issues.

## 0.12.0 (18. July, 2024)
### Changed
- (Breaking) Updated to Wgpu and Naga v22.0.0

## 0.11.2 (16. July, 2024)
### Fixed
- Readme discord badge added.

## 0.11.1 (15. July, 2024)
### Fixed
- Readme image issue on crates.io

## 0.11.0 (15. July, 2024)
### Added
- Defragmentation to Atlas_set. Reduces Texture Fragmented DeadSpace allowing more textures to fit later.
- with_deallocations_limit to Atlas_set to allow setting a custom defrag ratio.

### Changed
- (Breaking) Added size that is clamped to 256 to limits.max_texture_dimension_2d.
- (Breaking) Removed TextureView from AtlasSet and placed into TextureGroup.
- Added Backend to GpuRenderer 

### Fixed
- Reduced Textures loaded to 1 if not Opengl being used as an adapter backend.
- Rebinding TextureGroup upon Grow allows Access to new Texture Layers.

### Added
- GpuRenderer::get_layout() to acquire already made layouts without the need for a &mut GpuRenderer. 

## 0.10.1 (25. June, 2024)
### Changed
- Added Feature PassThru to Give direct access to all usable internal crates.

### Fixed
- Atlas Grow had the wrong Format and mip_level_count incorrect. 

## 0.10.0 (19. June, 2024)
### Changed
- (Breaking) Removed Visible lines function in Text as Cosmic removed it as well.
- (Breaking) Updated to cosmic-text 0.12.0

### Added
- Text::visible_details() which will return the needed details to calculate the render texts size.

## 0.9.0 (6. June, 2024)
### Changed
- (Breaking) Added Rendering layer to Text, Mesh.
- (Breaking) DrawOrder Width, Height and DrawType Removed. 
- (Breaking) DrawType Removed.
- (Breaking) Rename GpuBuffer as VertexBuffer.
- (Breaking) Bounds functions now use Vec2 instead of Vec3 since we do not use Z.
- (Breaking) Removed tex_buf from Text to reduce Ram usage.

### Fixed
- Ensure all renderers use Alpha checks and Rendering Layer for Ordering

### Added
- More Documentation.

## 0.8.2 (30. May, 2024)
### Fixed
- sRGB to Linear color within shader to give same or closer results to Paint and other editing programs that use RGB

## 0.8.1 (17. May, 2024)
### Fixed
- fixed Y ordering offsets.

## 0.8.0 (24. April, 2024)
### Changed
- (Breaking) updated to support winit 0.30.0.
- (Breaking) renderer update now takes &WindowEvent.

### Added
- Z axis angle Rotation and Flip to image.

## 0.7.0 (24. April, 2024)
### Changed
- (Breaking) system not supports a secondary manual Mat4x4 and Scale.
- (Breaking) use_camera is now set as camera_type and uses a enumeration which tells the shader how to use the camera's.
- (Breaking) projected_world_to_screen and world_to_screen now both use CameraType instead of scale.
- (Breaking) shaders were rewritten to allow multiple views and scales.

### Fixed
- rect not rendering correctly due to scale was always being applied even when view was not.


## 0.5.0 (24. April, 2024)
### Changed
- (Breaking)  BufferStore::new now expects sizes for vertex and index's Vec

### Fixed
- Updated Rendering types to have persistent data to avoid recreation upon update, which avoids allocations.
