use std::{
    error::Error,
    io::stdout,
    io::Write,
    ops::{Deref, DerefMut},
};

use crossterm::{
    cursor, execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType, SetSize},
};

use crate::{
    core::{
        data::{Dimensions2d, Layer, Matrix},
        BehaviourList, Entity, TerminalRenderable,
    },
    get_behaviour_name,
};

use super::Renderer;

const HORIZONTAL_OUTLINE_DELIMITER: &str = "=";
const VERTICAL_OUTLINE_DELIMITER: &str = "|";
const NEWLINE_DELIMITER: &str = "\r\n";

pub struct TerminalRendererConfig {
    pub screen_resolution: Dimensions2d,
    pub include_screen_outline: bool,
}

pub struct TerminalRenderer {
    initial_terminal_size: (u16, u16),
    config: TerminalRendererConfig,
    prev_render: String,
    is_initial_render: bool,
}
impl TerminalRenderer {
    pub fn new(config: TerminalRendererConfig) -> Self {
        TerminalRenderer {
            initial_terminal_size: (0, 0),
            config,
            prev_render: String::new(),
            is_initial_render: true,
        }
    }

    fn make_render_matrix(
        &self,
        entities: &Vec<(&Entity, &BehaviourList)>,
    ) -> TerminalRendererMatrix {
        let mut render_matrix = TerminalRendererMatrix::new(self.config.screen_resolution.clone());

        entities
            .iter()
            .filter_map(|(entity, behaviours)| {
                if let Some(terminal_renderable_behaviour) = behaviours
                    .get_behaviour::<TerminalRenderable>(get_behaviour_name!(TerminalRenderable))
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
                        if layer.is_above(&cell.data().layer_of_value)
                            || layer.is_with(&cell.data().layer_of_value)
                        {
                            render_matrix.update_cell_at(
                                x,
                                y,
                                TerminalRendererMatrixCell {
                                    display: *display,
                                    layer_of_value: layer.clone(),
                                },
                            );
                        }
                    }
                }
            });

        render_matrix
    }

    fn outline_render_string(&self, render_string: String) -> String {
        let make_horizontal_outline = || -> String {
            (0..self.config.screen_resolution.width())
                .map(|_| HORIZONTAL_OUTLINE_DELIMITER)
                .collect::<Vec<&str>>()
                .join("")
                .to_string()
        };

        let header = format!("/{}\\", make_horizontal_outline());
        let footer = format!("\\{}/", make_horizontal_outline());

        let body = render_string
            .split(NEWLINE_DELIMITER)
            .map(|line| format!("{VERTICAL_OUTLINE_DELIMITER}{line}{VERTICAL_OUTLINE_DELIMITER}"))
            .collect::<Vec<String>>()
            .join(NEWLINE_DELIMITER);

        format!("{header}{NEWLINE_DELIMITER}{body}{NEWLINE_DELIMITER}{footer}")
    }

    fn produce_render_string(&self, entities: &Vec<(&Entity, &BehaviourList)>) -> String {
        let render_matrix = self.make_render_matrix(&entities);

        let mut render_string = String::new();

        let mut curr_row = 0;
        for cell in render_matrix.iter() {
            let (_, row) = cell.location().values();
            let is_new_row = row != curr_row;

            if is_new_row {
                render_string += NEWLINE_DELIMITER;
            }

            render_string += &cell.data().display.to_string();

            curr_row = row;
        }

        format!(
            "{}{}{}",
            NEWLINE_DELIMITER,
            if self.config.include_screen_outline {
                self.outline_render_string(render_string)
            } else {
                render_string
            },
            NEWLINE_DELIMITER
        )
    }
}

impl Renderer for TerminalRenderer {
    fn init(&mut self) {
        if let Ok(size) = terminal::size() {
            self.initial_terminal_size = size;
        } else {
            panic!("TerminalRenderer could not get the terminal's starting size.");
        }

        if self.config.screen_resolution.height() > u16::MAX as u64
            || self.config.screen_resolution.width() > u16::MAX as u64
        {
            panic!("TerminalRenderer's screen resolution is too large. Neither the width nor height can be greater than {}", u16::MAX);
        }

        if let Err(e) = execute!(
            stdout(),
            Clear(ClearType::All),
            SetSize(
                self.config.screen_resolution.width() as u16,
                self.config.screen_resolution.height() as u16
            ),
            cursor::Hide,
            cursor::MoveTo(0, 0),
        ) {
            panic!(
                "TerminalRenderer could not do initial setup of game screen. Error: {}",
                e
            );
        }

        if let Err(e) = enable_raw_mode() {
            panic!(
                "TerminalRenderer could not set raw mode, cannot continue. Error: {}",
                e
            );
        }
    }

    fn render(&mut self, entities: Vec<(&Entity, &BehaviourList)>) -> Result<(), Box<dyn Error>> {
        let new_render_string = self.produce_render_string(&entities);

        if self.is_initial_render {
            write!(stdout(), "{}", new_render_string)?;

            self.is_initial_render = false;
        } else {
            let new_render_lines = new_render_string
                .split(NEWLINE_DELIMITER)
                .collect::<Vec<&str>>();
            let prev_render_lines = self
                .prev_render
                .split(NEWLINE_DELIMITER)
                .collect::<Vec<&str>>();

            for row in 0..new_render_lines.len() {
                if new_render_lines[row] != prev_render_lines[row] {
                    execute!(stdout(), cursor::MoveTo(0, row as u16))?;
                    write!(stdout(), "{}", new_render_lines[row].replace("\r\n", ""))?;
                    execute!(stdout(), Clear(ClearType::UntilNewLine))?;
                }
            }
        }

        self.prev_render = new_render_string;

        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), Box<dyn Error>> {
        execute!(
            stdout(),
            SetSize(self.initial_terminal_size.0, self.initial_terminal_size.1),
            cursor::MoveTo(0, self.initial_terminal_size.1),
            cursor::Show,
        )?;

        disable_raw_mode()?;

        Ok(())
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
    layer_of_value: Layer,
}
impl TerminalRendererMatrixCell {
    fn default() -> Self {
        Self {
            display: ' ',
            layer_of_value: Layer::base(),
        }
    }
}

fn is_entity_on_screen(entity: &Entity) -> bool {
    entity.transform().coords().x() >= 0.0 && entity.transform().coords().y() >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    mod produce_draw_string {
        use super::*;

        mod no_screen_outline {
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
                        BehaviourList::from(vec![Box::new(TerminalRenderable::new(
                            '^',
                            Layer::base(),
                        ))]),
                    ),
                    (
                        Entity::new("E3", Transform::new(Coords::new(0.0, 0.0, 0.0))),
                        BehaviourList::from(vec![Box::new(TerminalRenderable::new(
                            '5',
                            Layer::base(),
                        ))]),
                    ),
                    (
                        Entity::new("E4", Transform::default()),
                        BehaviourList::new(),
                    ),
                    (
                        Entity::new("E5", Transform::new(Coords::new(2.0, 2.0, 0.0))),
                        BehaviourList::from(vec![Box::new(TerminalRenderable::new(
                            '@',
                            Layer::base(),
                        ))]),
                    ),
                ];

                let result = renderer.produce_render_string(
                    &list
                        .iter()
                        .map(|(e, b)| (e, b))
                        .collect::<Vec<(&Entity, &BehaviourList)>>(),
                );

                assert_eq!(result, "\r\n5  \r\n ^ \r\n  @\r\n")
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
                        BehaviourList::from(vec![Box::new(TerminalRenderable::new(
                            '^',
                            Layer::new(1),
                        ))]),
                    ),
                    (
                        Entity::new("E3", Transform::new(Coords::new(0.0, 0.0, 0.0))),
                        BehaviourList::from(vec![Box::new(TerminalRenderable::new(
                            '5',
                            Layer::base(),
                        ))]),
                    ),
                    (
                        Entity::new("E4", Transform::default()),
                        BehaviourList::new(),
                    ),
                    (
                        Entity::new("E5", Transform::new(Coords::new(2.0, 2.0, 0.0))),
                        BehaviourList::from(vec![Box::new(TerminalRenderable::new(
                            '@',
                            Layer::base(),
                        ))]),
                    ),
                ];

                let result = renderer.produce_render_string(
                    &list
                        .iter()
                        .map(|(e, b)| (e, b))
                        .collect::<Vec<(&Entity, &BehaviourList)>>(),
                );

                assert_eq!(result, "\r\n5  \r\n   \r\n  ^\r\n")
            }
        }

        mod with_screen_outline {
            use crate::core::data::{Coords, Transform};

            use super::*;

            #[test]
            fn it_includes_all_renderable_entities() {
                let renderer = TerminalRenderer::new(TerminalRendererConfig {
                    screen_resolution: Dimensions2d::new(3, 3),
                    include_screen_outline: true,
                });

                let list: Vec<(Entity, BehaviourList)> = vec![
                    (
                        Entity::new("E1", Transform::default()),
                        BehaviourList::new(),
                    ),
                    (
                        Entity::new("E2", Transform::new(Coords::new(1.0, 1.0, 0.0))),
                        BehaviourList::from(vec![Box::new(TerminalRenderable::new(
                            '^',
                            Layer::base(),
                        ))]),
                    ),
                    (
                        Entity::new("E3", Transform::new(Coords::new(0.0, 0.0, 0.0))),
                        BehaviourList::from(vec![Box::new(TerminalRenderable::new(
                            '5',
                            Layer::base(),
                        ))]),
                    ),
                    (
                        Entity::new("E4", Transform::default()),
                        BehaviourList::new(),
                    ),
                    (
                        Entity::new("E5", Transform::new(Coords::new(2.0, 2.0, 0.0))),
                        BehaviourList::from(vec![Box::new(TerminalRenderable::new(
                            '@',
                            Layer::base(),
                        ))]),
                    ),
                ];

                let result = renderer.produce_render_string(
                    &list
                        .iter()
                        .map(|(e, b)| (e, b))
                        .collect::<Vec<(&Entity, &BehaviourList)>>(),
                );

                assert_eq!(
                    result,
                    "\r\n/===\\\r\n|5  |\r\n| ^ |\r\n|  @|\r\n\\===/\r\n"
                );
            }

            #[test]
            fn values_on_higher_layer_overwrite_lower_layer_values() {
                let renderer = TerminalRenderer::new(TerminalRendererConfig {
                    screen_resolution: Dimensions2d::new(3, 3),
                    include_screen_outline: true,
                });

                let list: Vec<(Entity, BehaviourList)> = vec![
                    (
                        Entity::new("E1", Transform::default()),
                        BehaviourList::new(),
                    ),
                    (
                        Entity::new("E2", Transform::new(Coords::new(2.0, 2.0, 0.0))),
                        BehaviourList::from(vec![Box::new(TerminalRenderable::new(
                            '^',
                            Layer::new(1),
                        ))]),
                    ),
                    (
                        Entity::new("E3", Transform::new(Coords::new(0.0, 0.0, 0.0))),
                        BehaviourList::from(vec![Box::new(TerminalRenderable::new(
                            '5',
                            Layer::base(),
                        ))]),
                    ),
                    (
                        Entity::new("E4", Transform::default()),
                        BehaviourList::new(),
                    ),
                    (
                        Entity::new("E5", Transform::new(Coords::new(2.0, 2.0, 0.0))),
                        BehaviourList::from(vec![Box::new(TerminalRenderable::new(
                            '@',
                            Layer::base(),
                        ))]),
                    ),
                ];

                let result = renderer.produce_render_string(
                    &list
                        .iter()
                        .map(|(e, b)| (e, b))
                        .collect::<Vec<(&Entity, &BehaviourList)>>(),
                );

                assert_eq!(
                    result,
                    "\r\n/===\\\r\n|5  |\r\n|   |\r\n|  ^|\r\n\\===/\r\n"
                );
            }
        }
    }
}
