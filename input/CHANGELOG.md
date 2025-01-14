# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
## Unreleased

## 0.4.1 (14. Janurary, 2025)
### Fixed
- Mouse Scroll Returning only 1 no matter what.

### Changed
- Device Events now only Trigger a Rerender other than MouseMotion which handles mouse delta and a trigger for render.

## 0.4.0 (13. Janurary, 2025)
### Changed
- (Breaking) Removed Mouse Position that was Calulated using hidpi. If you need this you can get it yourself by Position.x * HiDPI.
- (Breaking) Pysical Mouse Position is now Mouse Position.
- (Breaking) MouseWheel now returns Data.
- (Breaking) MousePosition now returns Position.
- (Breaking) Device Inputs are now Directly inserted into input_events.
- (Breaking) input_events is now a VecDeque
- (Breaking) events() now  returns a VecDeque<InputEvent> and Clears old Inputs.

### Added
- pop_event() to allow getting the next event in a timely fashion. Prefer this over events().

## 0.3.2 (9. Janurary, 2025)
### Added
- Get and Set for Click Duration.

## 0.3.1 (16. July, 2024)
### Changed
- ReadMe updates
- Added Fix for NumberPad Numlock output.

## 0.3.0 (5. May, 2024)
### Changed
- (Breaking) Updated to winit 0.30.0.
- (Breaking) Split InputHandler update into window_updates and device_updates
- (Breaking) window_updates handles all input returns even those from device_updates.

