use std::{
    io::stdout,
    io::Write,
    ops::{Deref, DerefMut},
};

use crossterm::{
    cursor, execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType, SetSize},
};

use crate::{
    Component, Dimensions2d, Layer, Matrix, Priority, Query, QueryResultList, System,
    TerminalRenderer, TerminalTransform,
};

const HORIZONTAL_OUTLINE_DELIMITER: &str = "=";
const VERTICAL_OUTLINE_DELIMITER: &str = "|";
const NEWLINE_DELIMITER: &str = "\r\n";

const TERMINAL_DIMENSIONS_PADDING: u16 = 10;

#[derive(Component, Debug)]
pub(crate) struct TerminalRendererState {
    pub initial_terminal_size: (u16, u16),
    pub options: TerminalRendererOptions,
    pub prev_render: String,
    pub is_initial_render: bool,
}
impl TerminalRendererState {
    pub(crate) fn new(options: TerminalRendererOptions) -> Self {
        TerminalRendererState {
            initial_terminal_size: (0, 0),
            options,
            prev_render: String::new(),
            is_initial_render: true,
        }
    }
}

pub(crate) struct TerminalRendererSystems {
    init_system: System,
    update_system: System,
    cleanup_system: System,
}
impl TerminalRendererSystems {
    pub(crate) fn new(options: TerminalRendererOptions) -> Self {
        Self {
            init_system: System::new_with_priority(
                Priority::highest(),
                vec![Query::new().include::<TerminalRendererState>()],
                move |results, _| {
                    if let [state_query, ..] = &results[..] {
                        assert!(
                            state_query.inclusions().len() == 1,
                            "There must be exactly 1 {} in the game. Found {}",
                            TerminalRendererState::name(),
                            state_query.inclusions().len()
                        );

                        let mut state = state_query
                            .inclusions()
                            .get(0)
                            .expect(&format!(
                                "There is a {} available in the world at init.",
                                TerminalRendererState::name()
                            ))
                            .components()
                            .get_mut::<TerminalRendererState>();

                        if let Ok(size) = terminal::size() {
                            state.initial_terminal_size = size;
                        } else {
                            panic!("TerminalRenderer could not get the terminal's starting size.");
                        }

                        if state.options.screen_resolution.height()
                            + TERMINAL_DIMENSIONS_PADDING as u64
                            > u16::MAX as u64
                            || state.options.screen_resolution.width()
                                + TERMINAL_DIMENSIONS_PADDING as u64
                                > u16::MAX as u64
                        {
                            panic!("TerminalRenderer's screen resolution is too large. Neither the width nor height can be greater than {}", u16::MAX - TERMINAL_DIMENSIONS_PADDING);
                        }

                        if let Err(e) = execute!(
                            stdout(),
                            Clear(ClearType::All),
                            SetSize(
                                state.options.screen_resolution.width() as u16
                                    + TERMINAL_DIMENSIONS_PADDING,
                                state.options.screen_resolution.height() as u16
                                    + TERMINAL_DIMENSIONS_PADDING
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
                },
            ),
            update_system: System::new_with_priority(
                Priority::higher_than(Priority::lowest()),
                vec![Query::new()
                    .has::<TerminalRenderer>()
                    .has_where::<TerminalTransform>(move |transform_terminal| {
                        let (x, y) = transform_terminal.coords.values();

                        (x >= 0 && x as u64 <= options.screen_resolution.width())
                            && (y >= 0 && y as u64 <= options.screen_resolution.height())
                    })
                    .include::<TerminalRendererState>()],
                move |results, _| {
                    if let [renderables_query, state_query, ..] = &results[..] {
                        let mut state = state_query
                            .inclusions()
                            .get(0)
                            .expect(&format!(
                                "The {} component is available on update.",
                                TerminalRendererState::name()
                            ))
                            .components()
                            .get_mut::<TerminalRendererState>();

                        let new_render_string = make_render_string(&renderables_query, &state.options);

                        if state.is_initial_render {
                            if let Err(e) = write!(stdout(), "{}", new_render_string) {
                                panic!("Error occurred while trying to write initial render to the terminal: {e}");
                            }

                            state.is_initial_render = false;
                        } else {
                            let new_render_lines = new_render_string
                                .split(NEWLINE_DELIMITER)
                                .collect::<Vec<&str>>();
                            let prev_render_lines = state
                                .prev_render
                                .split(NEWLINE_DELIMITER)
                                .collect::<Vec<&str>>();

                            for row in 0..new_render_lines.len() {
                                if new_render_lines[row] != prev_render_lines[row] {
                                    if let Err(e) =
                                        execute!(stdout(), cursor::MoveTo(0, row as u16))
                                    {
                                        panic!("Error occurred while trying to move the cursor to position (0, {}): {e}", row as u16);
                                    }

                                    if let Err(e) = write!(
                                        stdout(),
                                        "{}",
                                        new_render_lines[row].replace("\r\n", "")
                                    ) {
                                        panic!("Error occurred while trying to write the new render to the terminal: {e}");
                                    }

                                    if let Err(e) =
                                        execute!(stdout(), Clear(ClearType::UntilNewLine))
                                    {
                                        panic!("Error occurred while trying to execute the UntilNewLine clear type: {e}");
                                    }
                                }
                            }
                        }

                        state.prev_render = new_render_string;
                    }
                },
            ),
            cleanup_system: System::new(
                vec![Query::new().include::<TerminalRendererState>()],
                |results, _| {
                    if let [state_query, ..] = &results[..] {
                        let state = state_query
                            .inclusions()
                            .get(0)
                            .expect(&format!(
                                "The {} component is available on cleanup.",
                                TerminalRendererState::name()
                            ))
                            .components()
                            .get::<TerminalRendererState>();

                        let error_message =
                        "The terminal may be in a bad state. It's recommended to restart it if you intend to continue using this terminal instance.";

                        if let Err(e) = execute!(
                            stdout(),
                            SetSize(state.initial_terminal_size.0, state.initial_terminal_size.1),
                            cursor::MoveTo(0, state.initial_terminal_size.1),
                            cursor::Show,
                        ) {
                            println!("Could not reset terminal size and cursor visibility. {error_message} Error: {e}");
                        }

                        if let Err(e) = disable_raw_mode() {
                            println!("Could not disable raw mode. {error_message} Error: {e}");
                        }
                    }
                },
            ),
        }
    }

    pub(crate) fn extract_systems(self) -> (System, System, System) {
        (self.init_system, self.update_system, self.cleanup_system)
    }
}

fn make_render_matrix(
    renderables_query_result: &QueryResultList,
    renderer_options: &TerminalRendererOptions,
) -> TerminalRendererMatrix {
    let mut render_matrix = TerminalRendererMatrix::new(renderer_options.screen_resolution);

    for result in renderables_query_result {
        let (TerminalRenderer { display, layer }, coords) = (
            &*result.components().get::<TerminalRenderer>(),
            result.components().get::<TerminalTransform>().coords,
        );

        let (x, y) = (coords.x() as u64, coords.y() as u64);

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

    render_matrix
}

fn make_render_string(
    renderables_query_result: &QueryResultList,
    renderer_options: &TerminalRendererOptions,
) -> String {
    let render_matrix = make_render_matrix(renderables_query_result, &renderer_options);

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
        if renderer_options.include_screen_outline {
            outline_render_string(render_string, &renderer_options)
        } else {
            render_string
        },
        NEWLINE_DELIMITER
    )
}

fn outline_render_string(
    render_string: String,
    renderer_options: &TerminalRendererOptions,
) -> String {
    let make_horizontal_outline = || -> String {
        (0..renderer_options.screen_resolution.width())
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
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct TerminalRendererOptions {
    pub screen_resolution: Dimensions2d,
    pub include_screen_outline: bool,
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
            layer_of_value: Layer::furthest_background(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_make_draw_string {
        use super::*;

        mod no_screen_outline {
            use crate::{Entity, EntityManager, IntCoords2d};

            use super::*;

            #[test]
            fn it_includes_all_renderable_entities() {
                let options = TerminalRendererOptions {
                    screen_resolution: Dimensions2d::new(3, 3),
                    include_screen_outline: false,
                };

                let (_, update_system, _) = TerminalRendererSystems::new(options).extract_systems();

                let mut em = EntityManager::new();
                em.add_entity(
                    Entity(1),
                    vec![Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    })],
                );
                em.add_entity(
                    Entity(2),
                    vec![
                        Box::new(TerminalRenderer {
                            display: '^',
                            layer: Layer::base(),
                        }),
                        Box::new(TerminalTransform {
                            coords: IntCoords2d::new(1, 1),
                        }),
                    ],
                );
                em.add_entity(
                    Entity(3),
                    vec![
                        Box::new(TerminalRenderer {
                            display: '5',
                            layer: Layer::base(),
                        }),
                        Box::new(TerminalTransform {
                            coords: IntCoords2d::new(0, 0),
                        }),
                    ],
                );
                em.add_entity(
                    Entity(4),
                    vec![Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    })],
                );
                em.add_entity(
                    Entity(5),
                    vec![
                        Box::new(TerminalRenderer {
                            display: '@',
                            layer: Layer::base(),
                        }),
                        Box::new(TerminalTransform {
                            coords: IntCoords2d::new(2, 2),
                        }),
                    ],
                );

                let query_results = em.query(&update_system.queries()[0]);

                let result = make_render_string(&query_results, &options);

                assert_eq!(result, "\r\n5  \r\n ^ \r\n  @\r\n")
            }

            #[test]
            fn values_on_higher_layer_overwrite_lower_layer_values() {
                let options = TerminalRendererOptions {
                    screen_resolution: Dimensions2d::new(3, 3),
                    include_screen_outline: false,
                };

                let (_, update_system, _) = TerminalRendererSystems::new(options).extract_systems();

                let mut em = EntityManager::new();
                em.add_entity(
                    Entity(1),
                    vec![Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    })],
                );
                em.add_entity(
                    Entity(2),
                    vec![
                        Box::new(TerminalRenderer {
                            display: '^',
                            layer: Layer::new(1),
                        }),
                        Box::new(TerminalTransform {
                            coords: IntCoords2d::new(2, 2),
                        }),
                    ],
                );
                em.add_entity(
                    Entity(3),
                    vec![
                        Box::new(TerminalRenderer {
                            display: '5',
                            layer: Layer::base(),
                        }),
                        Box::new(TerminalTransform {
                            coords: IntCoords2d::new(0, 0),
                        }),
                    ],
                );
                em.add_entity(
                    Entity(4),
                    vec![Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    })],
                );
                em.add_entity(
                    Entity(5),
                    vec![
                        Box::new(TerminalRenderer {
                            display: '@',
                            layer: Layer::base(),
                        }),
                        Box::new(TerminalTransform {
                            coords: IntCoords2d::new(2, 2),
                        }),
                    ],
                );

                let query_results = em.query(&update_system.queries()[0]);

                let result = make_render_string(&query_results, &options);

                assert_eq!(result, "\r\n5  \r\n   \r\n  ^\r\n")
            }
        }

        mod with_screen_outline {
            use crate::{Entity, EntityManager, IntCoords2d};

            use super::*;

            #[test]
            fn it_includes_all_renderable_entities() {
                let options = TerminalRendererOptions {
                    screen_resolution: Dimensions2d::new(3, 3),
                    include_screen_outline: true,
                };

                let (_, update_system, _) = TerminalRendererSystems::new(options).extract_systems();

                let mut em = EntityManager::new();
                em.add_entity(
                    Entity(1),
                    vec![Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    })],
                );
                em.add_entity(
                    Entity(2),
                    vec![
                        Box::new(TerminalRenderer {
                            display: '^',
                            layer: Layer::base(),
                        }),
                        Box::new(TerminalTransform {
                            coords: IntCoords2d::new(1, 1),
                        }),
                    ],
                );
                em.add_entity(
                    Entity(3),
                    vec![
                        Box::new(TerminalRenderer {
                            display: '5',
                            layer: Layer::base(),
                        }),
                        Box::new(TerminalTransform {
                            coords: IntCoords2d::new(0, 0),
                        }),
                    ],
                );
                em.add_entity(
                    Entity(4),
                    vec![Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    })],
                );
                em.add_entity(
                    Entity(5),
                    vec![
                        Box::new(TerminalRenderer {
                            display: '@',
                            layer: Layer::base(),
                        }),
                        Box::new(TerminalTransform {
                            coords: IntCoords2d::new(2, 2),
                        }),
                    ],
                );

                let query_results = em.query(&update_system.queries()[0]);

                let result = make_render_string(&query_results, &options);

                assert_eq!(
                    result,
                    "\r\n/===\\\r\n|5  |\r\n| ^ |\r\n|  @|\r\n\\===/\r\n"
                );
            }

            #[test]
            fn values_on_higher_layer_overwrite_lower_layer_values() {
                let options = TerminalRendererOptions {
                    screen_resolution: Dimensions2d::new(3, 3),
                    include_screen_outline: true,
                };

                let (_, update_system, _) = TerminalRendererSystems::new(options).extract_systems();

                let mut em = EntityManager::new();
                em.add_entity(
                    Entity(1),
                    vec![Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    })],
                );
                em.add_entity(
                    Entity(2),
                    vec![
                        Box::new(TerminalRenderer {
                            display: '^',
                            layer: Layer::new(1),
                        }),
                        Box::new(TerminalTransform {
                            coords: IntCoords2d::new(2, 2),
                        }),
                    ],
                );
                em.add_entity(
                    Entity(3),
                    vec![
                        Box::new(TerminalRenderer {
                            display: '5',
                            layer: Layer::base(),
                        }),
                        Box::new(TerminalTransform {
                            coords: IntCoords2d::new(0, 0),
                        }),
                    ],
                );
                em.add_entity(
                    Entity(4),
                    vec![Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    })],
                );
                em.add_entity(
                    Entity(5),
                    vec![
                        Box::new(TerminalRenderer {
                            display: '@',
                            layer: Layer::base(),
                        }),
                        Box::new(TerminalTransform {
                            coords: IntCoords2d::new(2, 2),
                        }),
                    ],
                );

                let query_results = em.query(&update_system.queries()[0]);

                let result = make_render_string(&query_results, &options);

                assert_eq!(
                    result,
                    "\r\n/===\\\r\n|5  |\r\n|   |\r\n|  ^|\r\n\\===/\r\n"
                );
            }
        }
    }
}
