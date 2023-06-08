use crate::Component;

/// Identifying information that can be used to give entities meaningful identifying factors.
#[derive(Component, Debug)]
pub struct Identity {
  pub id: String,
  pub name: String,
}
