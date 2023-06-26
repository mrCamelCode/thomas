# 0.2.4

## Breaking Changes

- Priority's convenience methods (`lower_than`, `higher_than`) now only borrow the other Priority
- Coords structures have had their names updated to always reflect the space they're meant to be used in. This impacts `Coords` and `IntCoords`, which are now `Coords3d` and `IntCoords3d`.
- `WorldText` no longer has `coords`. Attach a transform component (like `TerminalTransform`) to an entity with `WorldText` to specify where in the world it's drawn.

## Updates

- Minor improvements.
- `TerminalCollision`:
  - Added `is_collision_between`.
  - Added `get_entity_on_layer`.
  - Added `get_body_on_layer`.
- Improved FPS tracking of engine analysis.
- The terminal renderer system will now treat a `None` background color as transparency. The background color to be used in a particular rendered cell is determined by starting with the highest layered renderable in that cell and seeing if it has a background color. If it doesn't, the system looks through the layers from there until it hits a renderable with a background color. If it finds one, it'll use that color. If it doesn't (and there's no `default_background_color`), it'll use the Reset color, which is the terminal's default color.
  - This will help considerably when you have something with no background color that's in front of something _with_ a background color. The thing in the back will have its color show, giving more visual continuity to a scene!
- Added `Vector` aliases to the `Coords` structures. Conceptually, the two are the same, but it's the context that's king in determining what it's operating as. Now you have the freedom to use whichever semantically fits in the current context of usage!
- Added `GameCommand::TriggerEvent`. Use it to trigger an event that you can assign systems to.
- Added `EVENT_AFTER_INIT`.
- Added `UiAnchor::Middle`.

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
