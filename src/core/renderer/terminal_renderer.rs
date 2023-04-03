use std::vec;

use crate::{
    core::{
        data::{Dimensions2d, IntCoords},
        BehaviourList, Entity, TerminalRenderable,
    },
    get_behaviour_name,
};

use super::Renderer;

pub struct TerminalRendererConfig {
    screen_resolution: Dimensions2d,
}

pub struct TerminalRenderer {
    config: TerminalRendererConfig,
}

impl TerminalRenderer {
    pub fn new(config: TerminalRendererConfig) -> Self {
        TerminalRenderer { config }
    }

    fn draw_entities(&self, entities: Vec<(&Entity, &BehaviourList)>) {
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
                        if is_layer_above_other(*layer, cell.layer_of_value) {
                            render_matrix.update_cell_at(x, y, *display, *layer);
                        }
                    }
                }
            });
    }
}

impl Renderer for TerminalRenderer {
    fn render(&self, entities: Vec<(&Entity, &BehaviourList)>) {
        self.draw_entities(entities);
    }
}

struct TerminalRendererMatrix {
    matrix: Vec<Vec<TerminalRendererMatrixCell>>,
    dimensions: Dimensions2d,
}

impl TerminalRendererMatrix {
    fn new(dimensions: Dimensions2d) -> Self {
        let mut matrix = vec![];

        for row in 0..=dimensions.height() {
            matrix.push(vec![]);

            for column in 0..=dimensions.width() {
                matrix[row as usize].push(TerminalRendererMatrixCell {
                    value: ' ',
                    layer_of_value: 0,
                    location: IntCoords::new(row as i64, column as i64, 0),
                });
            }
        }

        TerminalRendererMatrix { matrix, dimensions }
    }

    fn get(&self, x: u64, y: u64) -> Option<&TerminalRendererMatrixCell> {
        if x < self.dimensions.width() && y < self.dimensions.height() {
            return Some(&self.matrix[x as usize][y as usize]);
        }

        None
    }

    fn update_cell_at(&mut self, x: u64, y: u64, value: char, layer: u8) {
        if x < self.dimensions.width() && y < self.dimensions.height() {
            let mut cell = &mut self.matrix[x as usize][y as usize];

            cell.value = value;
            cell.layer_of_value = layer;
        }
    }
}

struct TerminalRendererMatrixCell {
    value: char,
    layer_of_value: u8,
    location: IntCoords,
}

fn is_entity_on_screen(entity: &Entity) -> bool {
    entity.transform().coords().x() > 0.0 && entity.transform().coords().y() > 0.0
}

fn is_layer_above_other(layer: u8, other: u8) -> bool {
    other >= layer
}
