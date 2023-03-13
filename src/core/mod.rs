pub mod behaviours;
pub mod data;

mod entity;
mod scene;

pub use entity::*;
pub use scene::*;

mod game;
use self::game::Game;

// TODO: Chapter 16 should discuss how to resolve this error.
pub static GAME: Game = game::Game::new();