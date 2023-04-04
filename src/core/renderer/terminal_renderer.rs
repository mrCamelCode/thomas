use std::{
    io::stdout,
    io::Write,
    ops::{Deref, DerefMut},
};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

use crate::{
    core::{
        data::{Dimensions2d, Matrix},
        BehaviourList, Entity, TerminalRenderable,
    },
    get_behaviour_name,
};

use super::Renderer;

pub struct TerminalRendererConfig {
    screen_resolution: Dimensions2d,
    // TODO: Use
    include_screen_outline: bool,
}

pub struct TerminalRenderer {
    config: TerminalRendererConfig,
}
impl TerminalRenderer {
    pub fn new(config: TerminalRendererConfig) -> Self {
        TerminalRenderer { config }
    }

    fn produce_render_matrix(
        &self,
        entities: &Vec<(&Entity, &BehaviourList)>,
    ) -> TerminalRendererMatrix {
        let mut render_matrix = TerminalRendererMatrix::new(self.config.screen_resolution.clone());

        entities
            .iter()
            .filter_map(|(entity, behaviours)| {
                if let Some(terminal_renderable_behaviour) =
                    behaviours.get::<TerminalRenderable>(get_behaviour_name!(TerminalRenderable))
                {
                    Some((entity, terminal_renderable_behaviour))
                } else {
                    None
                }
            })
            .for_each(|(entity, terminal_renderable_behaviour)| {
                let position = entity.transform().coords().rounded();
                let (x, y) = (position.x() as u64, position.y() as u64);

                let TerminalRenderable { display, layer } = terminal_renderable_behaviour;

                if is_entity_on_screen(entity) {
                    if let Some(cell) = render_matrix.get(x, y) {
                        if is_layer_above_other(*layer, cell.data().layer_of_value) {
                            render_matrix.update_cell_at(
                                x,
                                y,
                                TerminalRendererMatrixCell {
                                    display: *display,
                                    layer_of_value: *layer,
                                },
                            );
                        }
                    }
                }
            });

        render_matrix
    }

    fn produce_draw_string(&self, entities: &Vec<(&Entity, &BehaviourList)>) -> String {
        let render_matrix = self.produce_render_matrix(&entities);

        let mut render_string = String::new();

        let mut curr_row = 0;
        for cell in render_matrix.iter() {
            let (_, row) = cell.location().values();
            let is_new_row = row != curr_row;

            if is_new_row {
                render_string += "\r\n";
            }

            render_string += &cell.data().display.to_string();

            curr_row = row;
        }

        render_string
    }
}

impl Renderer for TerminalRenderer {
    fn init(&self) {
        if let Err(e) = enable_raw_mode() {
            panic!(
                "TerminalRenderer could not set raw mode, cannot continue. Error: {}",
                e
            );
        }
    }

    fn render(&self, entities: Vec<(&Entity, &BehaviourList)>) {
        execute!(stdout(), Clear(ClearType::All));

        let draw_string = self.produce_draw_string(&entities);

        write!(stdout(), "{}", draw_string);
    }

    fn cleanup(&self) {
        disable_raw_mode();
    }
}

struct TerminalRendererMatrix {
    matrix: Matrix<TerminalRendererMatrixCell>,
}
impl TerminalRendererMatrix {
    fn new(dimensions: Dimensions2d) -> Self {
        Self {
            matrix: Matrix::new(dimensions, TerminalRendererMatrixCell::default),
        }
    }
}
impl Deref for TerminalRendererMatrix {
    type Target = Matrix<TerminalRendererMatrixCell>;

    fn deref(&self) -> &Self::Target {
        &self.matrix
    }
}
impl DerefMut for TerminalRendererMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.matrix
    }
}

struct TerminalRendererMatrixCell {
    display: char,
    layer_of_value: u8,
}
impl TerminalRendererMatrixCell {
    fn default() -> Self {
        Self {
            display: ' ',
            layer_of_value: 0,
        }
    }
}

fn is_entity_on_screen(entity: &Entity) -> bool {
    entity.transform().coords().x() >= 0.0 && entity.transform().coords().y() >= 0.0
}

fn is_layer_above_other(layer: u8, other: u8) -> bool {
    layer >= other
}

#[cfg(test)]
mod tests {
    use super::*;

    mod produce_draw_string {
        use crate::core::data::{Coords, Transform};

        use super::*;

        #[test]
        fn it_includes_all_renderable_entities() {
            let renderer = TerminalRenderer::new(TerminalRendererConfig {
                screen_resolution: Dimensions2d::new(3, 3),
                include_screen_outline: false,
            });

            let list: Vec<(Entity, BehaviourList)> = vec![
                (
                    Entity::new("E1", Transform::default()),
                    BehaviourList::new(),
                ),
                (
                    Entity::new("E2", Transform::new(Coords::new(1.0, 1.0, 0.0))),
                    BehaviourList::from(vec![Box::new(TerminalRenderable::new('^', 0))]),
                ),
                (
                    Entity::new("E3", Transform::new(Coords::new(0.0, 0.0, 0.0))),
                    BehaviourList::from(vec![Box::new(TerminalRenderable::new('5', 0))]),
                ),
                (
                    Entity::new("E4", Transform::default()),
                    BehaviourList::new(),
                ),
                (
                    Entity::new("E5", Transform::new(Coords::new(2.0, 2.0, 0.0))),
                    BehaviourList::from(vec![Box::new(TerminalRenderable::new('@', 0))]),
                ),
            ];

            let result = renderer.produce_draw_string(
                &list
                    .iter()
                    .map(|(e, b)| (e, b))
                    .collect::<Vec<(&Entity, &BehaviourList)>>(),
            );

            assert_eq!(result, "5  \r\n ^ \r\n  @")
        }

        #[test]
        fn values_on_higher_layer_overwrite_lower_layer_values() {
            let renderer = TerminalRenderer::new(TerminalRendererConfig {
                screen_resolution: Dimensions2d::new(3, 3),
                include_screen_outline: false,
            });

            let list: Vec<(Entity, BehaviourList)> = vec![
                (
                    Entity::new("E1", Transform::default()),
                    BehaviourList::new(),
                ),
                (
                    Entity::new("E2", Transform::new(Coords::new(2.0, 2.0, 0.0))),
                    BehaviourList::from(vec![Box::new(TerminalRenderable::new('^', 1))]),
                ),
                (
                    Entity::new("E3", Transform::new(Coords::new(0.0, 0.0, 0.0))),
                    BehaviourList::from(vec![Box::new(TerminalRenderable::new('5', 0))]),
                ),
                (
                    Entity::new("E4", Transform::default()),
                    BehaviourList::new(),
                ),
                (
                    Entity::new("E5", Transform::new(Coords::new(2.0, 2.0, 0.0))),
                    BehaviourList::from(vec![Box::new(TerminalRenderable::new('@', 0))]),
                ),
            ];

            let result = renderer.produce_draw_string(
                &list
                    .iter()
                    .map(|(e, b)| (e, b))
                    .collect::<Vec<(&Entity, &BehaviourList)>>(),
            );

            assert_eq!(result, "5  \r\n   \r\n  ^")
        }
    }
}
