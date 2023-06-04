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
    Component, Dimensions2d, GameCommand, IntCoords2d, Layer, Matrix, Priority, Query,
    QueryResultList, System, SystemsGenerator, TerminalCamera, TerminalRenderer, TerminalTransform,
    EVENT_AFTER_UPDATE, EVENT_CLEANUP, EVENT_INIT,
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

pub(crate) struct TerminalRendererSystemsGenerator {}
impl TerminalRendererSystemsGenerator {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
impl SystemsGenerator for TerminalRendererSystemsGenerator {
    fn generate(&self) -> Vec<(&'static str, System)> {
        vec![
            (
                EVENT_INIT,
                System::new_with_priority(
                    Priority::highest(),
                    vec![Query::new().has::<TerminalRendererState>()],
                    move |results, util| {
                        if let [state_query, ..] = &results[..] {
                            assert!(
                                state_query.len() == 1,
                                "There must be exactly 1 {} in the game. Found {}",
                                TerminalRendererState::name(),
                                state_query.len()
                            );

                            let mut state = state_query
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
                                panic!(
                                    "TerminalRenderer could not get the terminal's starting size."
                                );
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

                            if state.options.include_default_camera {
                                util.commands().issue(GameCommand::AddEntity(vec![
                                    Box::new(TerminalCamera {
                                        field_of_view: state.options.screen_resolution.clone(),
                                        is_main: true,
                                    }),
                                    Box::new(TerminalTransform {
                                        coords: IntCoords2d::zero(),
                                    }),
                                ]))
                            }
                        }
                    },
                ),
            ),
            (
                EVENT_AFTER_UPDATE,
                System::new_with_priority(
                    Priority::lowest(),
                    vec![
                        Query::new()
                            .has::<TerminalRenderer>()
                            .has::<TerminalTransform>(),
                        Query::new().has::<TerminalRendererState>(),
                        Query::new()
                            .has_where::<TerminalCamera>(|camera| camera.is_main)
                            .has::<TerminalTransform>(),
                    ],
                    move |results, _| {
                        if let [renderables_query, state_query, camera_query, ..] = &results[..] {
                            let mut state = state_query
                                .get(0)
                                .expect(&format!(
                                    "The {} component is available on update.",
                                    TerminalRendererState::name()
                                ))
                                .components()
                                .get_mut::<TerminalRendererState>();

                            if let Some(camera_result) = camera_query.get(0) {
                                let main_camera =
                                    camera_result.components().get::<TerminalCamera>();
                                let main_camera_transform =
                                    camera_result.components().get::<TerminalTransform>();

                                let new_render_string = make_render_string(
                                    &*main_camera,
                                    &*main_camera_transform,
                                    &renderables_query,
                                    &state.options,
                                );

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
                        }
                    },
                ),
            ),
            (
                EVENT_CLEANUP,
                System::new(
                    vec![Query::new().has::<TerminalRendererState>()],
                    |results, _| {
                        if let [state_query, ..] = &results[..] {
                            let state = state_query
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
                                SetSize(
                                    state.initial_terminal_size.0,
                                    state.initial_terminal_size.1
                                ),
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
            ),
        ]
    }
}

fn make_render_string(
    main_camera: &TerminalCamera,
    main_camera_transform: &TerminalTransform,
    renderables_query_result: &QueryResultList,
    renderer_options: &TerminalRendererOptions,
) -> String {
    let render_matrix = make_render_matrix(
        main_camera,
        main_camera_transform,
        renderables_query_result,
        &renderer_options,
    );

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

fn make_render_matrix(
    main_camera: &TerminalCamera,
    main_camera_transform: &TerminalTransform,
    renderables_query_result: &QueryResultList,
    renderer_options: &TerminalRendererOptions,
) -> TerminalRendererMatrix {
    let mut render_matrix = TerminalRendererMatrix::new(renderer_options.screen_resolution);

    for result in renderables_query_result {
        let renderable_transform = result.components().get::<TerminalTransform>();
        let renderable_screen_position = renderable_transform.coords - main_camera_transform.coords;

        let TerminalRenderer { display, layer } = &*result.components().get::<TerminalRenderer>();

        let (x, y) = (
            renderable_screen_position.x() as u64,
            renderable_screen_position.y() as u64,
        );

        if is_renderable_visible(
            main_camera,
            main_camera_transform,
            &*renderable_transform,
            &renderer_options.screen_resolution,
        ) {
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
    }

    render_matrix
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

fn is_renderable_visible(
    main_camera: &TerminalCamera,
    main_camera_transform: &TerminalTransform,
    renderable_transform: &TerminalTransform,
    screen_resolution: &Dimensions2d,
) -> bool {
    let renderable_world_position = renderable_transform.coords;
    let camera_world_position = main_camera_transform.coords;
    let renderable_screen_position = renderable_world_position - camera_world_position;

    let camera_min_visible_pos = camera_world_position;
    let camera_max_visible_pos = IntCoords2d::new(
        camera_world_position.x() + main_camera.field_of_view.width() as i64 - 1,
        camera_world_position.y() + main_camera.field_of_view.height() as i64 - 1,
    );

    let is_within_camera_view = (renderable_world_position.x() >= camera_min_visible_pos.x()
        && renderable_world_position.x() <= camera_max_visible_pos.x())
        && (renderable_world_position.y() >= camera_min_visible_pos.y()
            && renderable_world_position.y() <= camera_max_visible_pos.y());

    let is_within_screen_space = (renderable_screen_position.x() >= 0
        && renderable_screen_position.x() < screen_resolution.width() as i64)
        && (renderable_screen_position.y() >= 0
            && renderable_screen_position.y() < screen_resolution.height() as i64);

    is_within_camera_view && is_within_screen_space
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct TerminalRendererOptions {
    pub screen_resolution: Dimensions2d,
    pub include_screen_outline: bool,
    pub include_default_camera: bool,
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
            use crate::{EntityManager, IntCoords2d};

            use super::*;

            #[test]
            fn it_includes_all_renderable_entities() {
                let options = TerminalRendererOptions {
                    screen_resolution: Dimensions2d::new(3, 3),
                    include_screen_outline: false,
                    include_default_camera: false,
                };

                let generated_renderer_systems = TerminalRendererSystemsGenerator::new().generate();

                let after_update_system = &generated_renderer_systems
                    .iter()
                    .find(|(name, _)| *name == EVENT_AFTER_UPDATE)
                    .unwrap()
                    .1;

                let mut em = EntityManager::new();
                em.add_entity(vec![Box::new(TerminalTransform {
                    coords: IntCoords2d::new(0, 0),
                })]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '^',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(1, 1),
                    }),
                ]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '5',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    }),
                ]);
                em.add_entity(vec![Box::new(TerminalTransform {
                    coords: IntCoords2d::new(0, 0),
                })]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '@',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(2, 2),
                    }),
                ]);

                let query_results = em.query(&after_update_system.queries()[0]);

                let result = make_render_string(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(3, 3),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &query_results,
                    &options,
                );

                assert_eq!(result, "\r\n5  \r\n ^ \r\n  @\r\n")
            }

            #[test]
            fn values_on_higher_layer_overwrite_lower_layer_values() {
                let options = TerminalRendererOptions {
                    screen_resolution: Dimensions2d::new(3, 3),
                    include_screen_outline: false,
                    include_default_camera: false,
                };

                let generated_renderer_systems = TerminalRendererSystemsGenerator::new().generate();

                let after_update_system = &generated_renderer_systems
                    .iter()
                    .find(|(name, _)| *name == EVENT_AFTER_UPDATE)
                    .unwrap()
                    .1;

                let mut em = EntityManager::new();
                em.add_entity(vec![Box::new(TerminalTransform {
                    coords: IntCoords2d::new(0, 0),
                })]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '^',
                        layer: Layer::new(1),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(2, 2),
                    }),
                ]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '5',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    }),
                ]);
                em.add_entity(vec![Box::new(TerminalTransform {
                    coords: IntCoords2d::new(0, 0),
                })]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '@',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(2, 2),
                    }),
                ]);

                let query_results = em.query(&after_update_system.queries()[0]);

                let result = make_render_string(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(3, 3),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &query_results,
                    &options,
                );

                assert_eq!(result, "\r\n5  \r\n   \r\n  ^\r\n")
            }

            #[test]
            fn entities_outside_of_the_camera_fov_are_not_rendered() {
                let options = TerminalRendererOptions {
                    screen_resolution: Dimensions2d::new(3, 3),
                    include_screen_outline: false,
                    include_default_camera: false,
                };

                let generated_renderer_systems = TerminalRendererSystemsGenerator::new().generate();

                let after_update_system = &generated_renderer_systems
                    .iter()
                    .find(|(name, _)| *name == EVENT_AFTER_UPDATE)
                    .unwrap()
                    .1;

                let mut em = EntityManager::new();
                em.add_entity(vec![Box::new(TerminalTransform {
                    coords: IntCoords2d::new(0, 0),
                })]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '^',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(1, 1),
                    }),
                ]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '5',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    }),
                ]);
                em.add_entity(vec![Box::new(TerminalTransform {
                    coords: IntCoords2d::new(0, 0),
                })]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '@',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(2, 2),
                    }),
                ]);

                let query_results = em.query(&after_update_system.queries()[0]);

                let result = make_render_string(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(3, 3),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::new(2, 1),
                    },
                    &query_results,
                    &options,
                );

                assert_eq!(result, "\r\n   \r\n@  \r\n   \r\n")
            }
        }

        mod with_screen_outline {
            use crate::{EntityManager, IntCoords2d};

            use super::*;

            #[test]
            fn it_includes_all_renderable_entities() {
                let options = TerminalRendererOptions {
                    screen_resolution: Dimensions2d::new(3, 3),
                    include_screen_outline: true,
                    include_default_camera: false,
                };

                let generated_renderer_systems = TerminalRendererSystemsGenerator::new().generate();

                let after_update_system = &generated_renderer_systems
                    .iter()
                    .find(|(name, _)| *name == EVENT_AFTER_UPDATE)
                    .unwrap()
                    .1;

                let mut em = EntityManager::new();
                em.add_entity(vec![Box::new(TerminalTransform {
                    coords: IntCoords2d::new(0, 0),
                })]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '^',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(1, 1),
                    }),
                ]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '5',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    }),
                ]);
                em.add_entity(vec![Box::new(TerminalTransform {
                    coords: IntCoords2d::new(0, 0),
                })]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '@',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(2, 2),
                    }),
                ]);

                let query_results = em.query(&after_update_system.queries()[0]);

                let result = make_render_string(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(3, 3),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &query_results,
                    &options,
                );

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
                    include_default_camera: false,
                };

                let generated_renderer_systems = TerminalRendererSystemsGenerator::new().generate();

                let after_update_system = &generated_renderer_systems
                    .iter()
                    .find(|(name, _)| *name == EVENT_AFTER_UPDATE)
                    .unwrap()
                    .1;

                let mut em = EntityManager::new();
                em.add_entity(vec![Box::new(TerminalTransform {
                    coords: IntCoords2d::new(0, 0),
                })]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '^',
                        layer: Layer::new(1),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(2, 2),
                    }),
                ]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '5',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(0, 0),
                    }),
                ]);
                em.add_entity(vec![Box::new(TerminalTransform {
                    coords: IntCoords2d::new(0, 0),
                })]);
                em.add_entity(vec![
                    Box::new(TerminalRenderer {
                        display: '@',
                        layer: Layer::base(),
                    }),
                    Box::new(TerminalTransform {
                        coords: IntCoords2d::new(2, 2),
                    }),
                ]);

                let query_results = em.query(&after_update_system.queries()[0]);

                let result = make_render_string(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(3, 3),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &query_results,
                    &options,
                );

                assert_eq!(
                    result,
                    "\r\n/===\\\r\n|5  |\r\n|   |\r\n|  ^|\r\n\\===/\r\n"
                );
            }
        }
    }
}
