# Next
## Updates
  - UI is now positioned relative to the main camera's viewport.
  - Color is now supported! Use `Rgb` to define colors for the renderer to use for entities. Colors are optional, and the default foreground and background colors you gave the renderer on `start` are used if no value is provided for a particular component that supports color. If you didn't provide a default for either the foreground or background, the color is `Reset`, which just uses the terminal's default.
## Breaking Changes
  - Color was added! The following structures now have new properties you'll need to include:
    1. `Text`
    1. `TerminalRenderer`
    1. `TerminalRendererOptions`

# 0.2.0
- Overhauled architecture to use ECS.

# 0.1.0
- Initial release