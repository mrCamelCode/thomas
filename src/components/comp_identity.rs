use crate::Component;

#[derive(Component, Debug)]
pub struct Identity {
  pub id: String,
  pub name: String,
}
