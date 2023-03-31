use std::vec;

use crate::{
    core::{
        data::{Coords, Dimensions2d}, EntityBehaviourMap,
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

    fn draw_entities(&self, entity_behaviour_map: &EntityBehaviourMap) {
        let mut render_matrix = TerminalRendererMatrix::new(self.config.screen_resolution.clone());

        scene.entities().iter().for_each(|entity| {
            let renderable_info = entity
            .behaviours()
            .get::<TerminalRenderable>(get_behaviour_name!(TerminalRenderable))

            if entity
                .behaviours()
                .get(get_behaviour_name!(TerminalRenderable))
            {
                let position = entity.transform().coords().rounded();

                if let Some(cell) = render_matrix.get(position.x(), position.y()) {}
            }
        });
    }
}

impl Renderer for TerminalRenderer {
    fn render(&self, scene: &Scene) {
        self.draw_entities(scene);
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
                matrix[row].push(TerminalRendererMatrixCell {
                    value: ' ',
                    layer_of_value: 0,
                    location: Coords::new(row as f64, column as f64, 0.0),
                });
            }
        }

        TerminalRendererMatrix { matrix, dimensions }
    }

    fn get(&self, x: usize, y: usize) -> Option<TerminalRendererMatrixCell> {
        if x < self.dimensions.width() && y < self.dimensions.height() {
            return Some(self.matrix[x][y]);
        }

        None
    }

    fn update(&mut self, x: usize, y: usize, value: char, layer: u8) {
        if x < self.dimensions.width() && y < self.dimensions.height() {
            self.matrix[x][y].value = value;
            self.matrix[x][y].layer_of_value = layer;
        }
    }
}

struct TerminalRendererMatrixCell {
    value: char,
    layer_of_value: u8,
    location: Coords,
}
