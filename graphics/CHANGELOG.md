# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
## Unreleased

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
- (Breaking)  BufferStore::new now expects sizes for vertex and indexs Vec

### Fixed
- Updated Rendering types to have persistant data to avoid recreation upon update, which avoids allocations.
