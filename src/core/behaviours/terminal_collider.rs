use crate::core::Behaviour;
use thomas_derive::Behaviour;

use crate::core::CustomBehaviour;

#[derive(Behaviour, Clone)]
pub struct TerminalCollider {}
impl TerminalCollider {}
impl CustomBehaviour for TerminalCollider {}
