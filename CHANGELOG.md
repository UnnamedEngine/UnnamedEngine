## 0.0.1-060224d

- Changed identation from `4` `spaces` to `2` `spaces`

## 0.0.1-280124d

- Improved the header of all source files

## 0.0.1-291223d

- Added a internal event dispatcher for the engine
- Added new events `Shutdown` and `Resize`

## 0.0.1-281223d

- `Camera` got moved into a camera file

## 0.0.1-271223d

- Defined a versioning standard at `VERSIONING.md`
- Exposed `Engine` to `start()`, `update()` and `render()` when called from applications
- Added a header with proper description for all the files
- Added `stop()` to request graceful shutdown
- Added a `Event` enum to handle the events sent by the engine
- Moved the `ESC` to close feature to the applications

## 0.0.1-241223d

- `Update` and `Render` from upper applications are now properly called
- Added env_logger
- Window now has the name passed by the application

## 0.0.1-240923d
- Start of the project
- Added github actions to build and run tests on Windows, Ubuntu and MacOS
- Barebones entry point
- Server project
- Client project
- Added the branding
- Added `README.md`
- Added `LICENSE.md` (MIT)
- Added `CHANGELOG.md`
