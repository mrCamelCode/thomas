# Next
## Breaking Changes
  - Priority's convenience methods (`lower_than`, `higher_than`) now only borrow the other Priority 
## Updates
  - Minor improvements.
  - Added `TerminalCollision::is_collision_between` convenience function.
  - Improved FPS tracking of engine analysis.

# 0.2.3
## Updates
  - Minor improvements
  - UI rendering and collisions are now included by default in terminal applications.

# 0.2.2
## Bugfixes
  - The `TerminalRenderer` no longer gets confused when working with a moving camera, and it cleans up after itself better.
## Updates
- Added `is_any_key...` methods to `Input`.

# 0.2.1

## Updates
- Color is now supported! Use `Rgb` to define colors for the renderer to use for entities. Colors are optional, and the default foreground and background colors you gave the renderer on `start` are used if no value is provided for a particular component that supports color. If you didn't provide a default for either the foreground or background, the color is `Reset`, which just uses the terminal's default.

## Breaking Changes

- Color was added! The following structures now have new properties you'll need to include:
  1. `Text`
  1. `TerminalRenderer`
  1. `TerminalRendererOptions`
- `Text` is now positioned relative to the main camera's viewport. To use get text that's rendered at a fixed world position, use `WorldText`.

# 0.2.0

- Overhauled architecture to use ECS.

# 0.1.0

- Initial release
