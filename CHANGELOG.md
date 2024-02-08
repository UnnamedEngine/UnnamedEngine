## 0.0.1 - 08/02/2024

- Updated `env_logger` crate to `0.11.1`
- Updated `winit` crate to `0.29.10`
- Updated `wgpu` crate to `0.19.1`
- Removed unused `use`s
- Changed the `CHANGELOG.md` format
- Created `Viewport` to abstract windows creation and handling
- Moved window logic from `core/state.rs` to `renderer/viewport.rs`
- Converted `unwrap`s to `expect`

## 0.0.1 - 08/02/2024

- Moved more parts of the camera from `core/state.rs` to `core/camera.rs`
- Created `MiddlewareRenderer` to abstract rendering
- Moved rendering from `core/state.rs` to `renderer/middleware_renderer.rs`

## 0.0.1 - 06/02/2024

- Changed identation from `4` `spaces` to `2` `spaces`
- Improved building information at `README.md`
- Moved `CameraUniform` from `core/state.rs` to `renderer/camera.rs`

## 0.0.1 - 28/01/2024

- Improved the header of all source files

## 0.0.1 - 29/12/2023

- Added a internal event dispatcher for the engine
- Added new events `Shutdown` and `Resize`

## 0.0.1 - 28/12/2023

- `Camera` got moved into a camera file

## 0.0.1 - 27/12/2023

- Defined a versioning standard at `VERSIONING.md`
- Exposed `Engine` to `start()`, `update()` and `render()` when called from applications
- Added a header with proper description for all the files
- Added `stop()` to request graceful shutdown
- Added a `Event` enum to handle the events sent by the engine
- Moved the `ESC` to close feature to the applications

## 0.0.1 - 24/12/2023

- `Update` and `Render` from upper applications are now properly called
- Added env_logger
- Window now has the name passed by the application

## 0.0.1 - 24/09/2023
- Start of the project
- Added github actions to build and run tests on Windows, Ubuntu and MacOS
- Barebones entry point
- Server project
- Client project
- Added the branding
- Added `README.md`
- Added `LICENSE.md` (MIT)
- Added `CHANGELOG.md`
