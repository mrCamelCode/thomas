use crate::Component;

/// Internal marker component that tracks the entities in the world that are created by the UI rendering system.
#[derive(Component)]
pub(crate) struct TerminalTextCharacter {}