// use std::{
//     any::Any,
//     collections::HashMap,
//     sync::{Arc, Mutex},
// };

// pub static RENDERER_OPTIONS: Arc<Mutex<HashMap<&'static str, Box<dyn Any>>>> =
//     Arc::new(Mutex::new(HashMap::new()));

mod sys_terminal_renderer;
pub use sys_terminal_renderer::*;
