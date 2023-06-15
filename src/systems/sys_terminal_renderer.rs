use std::{
    io::stdout,
    ops::{Deref, DerefMut},
};

use crossterm::{
    cursor, execute,
    style::{Color, PrintStyledContent, ResetColor, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType, SetSize},
};

use crate::{
    Component, Dimensions2d, GameCommand, IntCoords2d, Layer, Matrix, Priority, Query,
    QueryResultList, Rgb, System, SystemsGenerator, TerminalCamera, TerminalRenderer,
    TerminalTransform, EVENT_AFTER_UPDATE, EVENT_CLEANUP, EVENT_INIT,
};

const TERMINAL_DIMENSIONS_PADDING: u16 = 0;

#[derive(Component, Debug)]
pub(crate) struct TerminalRendererState {
    initial_terminal_size: (u16, u16),
    pub options: TerminalRendererOptions,
    prev_render: Option<TerminalRendererMatrix>,
}
impl TerminalRendererState {
    pub(crate) fn new(options: TerminalRendererOptions) -> Self {
        TerminalRendererState {
            initial_terminal_size: (0, 0),
            options,
            prev_render: None,
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
                    move |results, commands| {
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
                                cursor::SavePosition,
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
                                commands.borrow_mut().issue(GameCommand::AddEntity(vec![
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
                        if let [renderables_results, state_results, main_camera_results, ..] =
                            &results[..]
                        {
                            let mut state = state_results.get_only_mut::<TerminalRendererState>();

                            if let Some(camera_result) = main_camera_results.get(0) {
                                let main_camera =
                                    camera_result.components().get::<TerminalCamera>();
                                let main_camera_transform =
                                    camera_result.components().get::<TerminalTransform>();

                                if main_camera.field_of_view.width()
                                    > state.options.screen_resolution.width()
                                    || main_camera.field_of_view.height()
                                        > state.options.screen_resolution.height()
                                {
                                    panic!("Main camera's field of view cannot exceed the screen resolution. FOV: W: {}, H: {} | Resolution: W: {}, H: {}",
                                        main_camera.field_of_view.width(),
                                        main_camera.field_of_view.height(),
                                        state.options.screen_resolution.width(),
                                        state.options.screen_resolution.height()
                                    );
                                }

                                state.prev_render = Some(draw(
                                    &*main_camera,
                                    &*main_camera_transform,
                                    &renderables_results,
                                    &state.options,
                                    &state.prev_render,
                                ));
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
                            "The terminal may be in a bad state. It's recommended you don't continue to use this terminal instance.";

                            if let Err(e) = execute!(
                                stdout(),
                                SetSize(
                                    state.initial_terminal_size.0,
                                    state.initial_terminal_size.1
                                ),
                                cursor::Show,
                                cursor::RestorePosition,
                                ResetColor,
                                Clear(ClearType::All),
                            ) {
                                println!("Could not reset terminal size and cursor visibility. {error_message} Error: {e}");
                            }

                            if let Err(e) = disable_raw_mode() {
                                println!("Could not disable raw mode. {error_message} Error: {e}");
                            }

                            println!("Thanks for playing a game powered by Thomas!");
                        }
                    },
                ),
            ),
        ]
    }
}

fn draw(
    main_camera: &TerminalCamera,
    main_camera_transform: &TerminalTransform,
    renderables_query_result: &QueryResultList,
    renderer_options: &TerminalRendererOptions,
    previous_render: &Option<TerminalRendererMatrix>,
) -> TerminalRendererMatrix {
    let new_render_matrix = make_render_matrix(
        main_camera,
        main_camera_transform,
        renderables_query_result,
        renderer_options,
    );

    for new_cell in &*new_render_matrix {
        let (x, y) = new_cell.location().values();

        let prev_cell = if let Some(prev_render) = previous_render {
            prev_render.get(x as u64, y as u64)
        } else {
            None
        };

        if prev_cell.is_none() || new_cell.data() != prev_cell.unwrap().data() {
            if let Err(e) = execute!(
                stdout(),
                cursor::MoveTo(x as u16, y as u16),
                PrintStyledContent(
                    String::from(new_cell.data().display)
                        .with(get_crossterm_color(
                            &new_cell.data().foreground_color,
                            &renderer_options.default_foreground_color
                        ))
                        .on(get_crossterm_color(
                            &new_cell.data().background_color,
                            &renderer_options.default_background_color
                        ))
                ),
            ) {
                panic!(
                    "Error occurred while trying to write at position ({}, {}): {e}",
                    x as u16, y as u16
                );
            }
        }
    }

    new_render_matrix
}

fn get_crossterm_color(color_option: &Option<Rgb>, default_color_option: &Option<Rgb>) -> Color {
    let color_to_use = if color_option.is_some() {
        color_option
    } else if default_color_option.is_some() {
        default_color_option
    } else {
        &None
    };

    if let Some(color) = color_to_use {
        Color::parse_ansi(&format!("2;{};{};{}", color.r(), color.g(), color.b()))
            .expect("Color is supported.")
    } else {
        Color::Reset
    }
}

fn make_render_matrix(
    main_camera: &TerminalCamera,
    main_camera_transform: &TerminalTransform,
    renderables_query_result: &QueryResultList,
    renderer_options: &TerminalRendererOptions,
) -> TerminalRendererMatrix {
    let mut render_matrix = TerminalRendererMatrix::new(
        main_camera.field_of_view,
        renderer_options.default_background_color,
        renderer_options.default_foreground_color,
    );

    for result in renderables_query_result {
        let renderable_transform = result.components().get::<TerminalTransform>();

        let TerminalRenderer {
            display,
            layer,
            foreground_color,
            background_color,
        } = &*result.components().get::<TerminalRenderer>();

        if is_renderable_visible(main_camera, main_camera_transform, &*renderable_transform) {
            let renderable_screen_position = convert_world_position_to_screen_position(
                main_camera_transform,
                &renderable_transform.coords,
            );
            let (x, y) = (
                renderable_screen_position.x() as u64,
                renderable_screen_position.y() as u64,
            );

            if let Some(cell) = render_matrix.get(x, y) {
                if layer.is_above(&cell.data().layer_of_value)
                    || layer.is_with(&cell.data().layer_of_value)
                {
                    render_matrix.update_cell_at(
                        x,
                        y,
                        TerminalRendererMatrixCell {
                            display: *display,
                            layer_of_value: *layer,
                            foreground_color: *foreground_color,
                            background_color: *background_color,
                        },
                    );
                }
            }
        }
    }

    render_matrix
}

fn is_renderable_visible(
    main_camera: &TerminalCamera,
    main_camera_transform: &TerminalTransform,
    renderable_transform: &TerminalTransform,
) -> bool {
    let screen_position = convert_world_position_to_screen_position(
        main_camera_transform,
        &renderable_transform.coords,
    );

    (screen_position.x() >= 0 && screen_position.x() < main_camera.field_of_view.width() as i64)
        && (screen_position.y() >= 0
            && screen_position.y() < main_camera.field_of_view.height() as i64)
}

fn convert_world_position_to_screen_position(
    main_camera_transform: &TerminalTransform,
    world_coords: &IntCoords2d,
) -> IntCoords2d {
    *world_coords - main_camera_transform.coords
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct TerminalRendererOptions {
    pub screen_resolution: Dimensions2d,
    pub include_default_camera: bool,
    pub default_foreground_color: Option<Rgb>,
    pub default_background_color: Option<Rgb>,
}

#[derive(Debug)]
struct TerminalRendererMatrix {
    matrix: Matrix<TerminalRendererMatrixCell>,
}
impl TerminalRendererMatrix {
    fn new(
        dimensions: Dimensions2d,
        default_background_color: Option<Rgb>,
        default_foreground_color: Option<Rgb>,
    ) -> Self {
        Self {
            matrix: Matrix::new(dimensions, || {
                TerminalRendererMatrixCell::default(
                    default_background_color,
                    default_foreground_color,
                )
            }),
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

#[derive(Debug, PartialEq, Eq)]
struct TerminalRendererMatrixCell {
    display: char,
    layer_of_value: Layer,
    foreground_color: Option<Rgb>,
    background_color: Option<Rgb>,
}
impl TerminalRendererMatrixCell {
    fn default(
        default_background_color: Option<Rgb>,
        default_foreground_color: Option<Rgb>,
    ) -> Self {
        Self {
            display: ' ',
            layer_of_value: Layer::furthest_background(),
            foreground_color: default_foreground_color,
            background_color: default_background_color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_convert_world_position_to_screen_position {
        use super::*;

        #[test]
        fn screen_positions_are_equivalent_with_no_camera_offset() {
            assert_eq!(
                convert_world_position_to_screen_position(
                    &TerminalTransform {
                        coords: IntCoords2d::zero()
                    },
                    &IntCoords2d::new(1, 2)
                ),
                IntCoords2d::new(1, 2)
            );
        }

        #[test]
        fn screen_positions_are_correct_with_camera_negative_x_offset() {
            assert_eq!(
                convert_world_position_to_screen_position(
                    &TerminalTransform {
                        coords: IntCoords2d::new(-3, 0)
                    },
                    &IntCoords2d::new(1, 2)
                ),
                IntCoords2d::new(4, 2)
            );
        }

        #[test]
        fn screen_positions_are_correct_with_camera_positive_x_offset() {
            assert_eq!(
                convert_world_position_to_screen_position(
                    &TerminalTransform {
                        coords: IntCoords2d::new(2, 0)
                    },
                    &IntCoords2d::new(1, 2)
                ),
                IntCoords2d::new(-1, 2)
            );
        }

        #[test]
        fn screen_positions_are_correct_with_camera_negative_y_offset() {
            assert_eq!(
                convert_world_position_to_screen_position(
                    &TerminalTransform {
                        coords: IntCoords2d::new(0, -2)
                    },
                    &IntCoords2d::new(1, 2)
                ),
                IntCoords2d::new(1, 4)
            );
        }

        #[test]
        fn screen_positions_are_correct_with_camera_positive_y_offset() {
            assert_eq!(
                convert_world_position_to_screen_position(
                    &TerminalTransform {
                        coords: IntCoords2d::new(0, 5)
                    },
                    &IntCoords2d::new(1, 2)
                ),
                IntCoords2d::new(1, -3)
            );
        }
    }

    mod test_get_crossterm_color {
        use super::*;

        #[test]
        fn color_code_is_correct_when_color_is_provided() {
            assert_eq!(
                get_crossterm_color(&Some(Rgb::white()), &None),
                Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255
                }
            );
        }

        #[test]
        fn color_code_is_correct_when_default_color_is_provided() {
            assert_eq!(
                get_crossterm_color(&Some(Rgb::white()), &Some(Rgb::black())),
                Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255
                }
            );
        }

        #[test]
        fn color_code_is_correct_when_only_default_color_is_provided() {
            assert_eq!(
                get_crossterm_color(&None, &Some(Rgb::black())),
                Color::Rgb { r: 0, g: 0, b: 0 }
            );
        }

        #[test]
        fn color_code_is_reset_when_no_colors_are_provided() {
            assert_eq!(get_crossterm_color(&None, &None), Color::Reset)
        }
    }

    mod test_terminal_renderer_matrix_cell_equality {
        use super::*;

        #[test]
        fn equivalent_cells_are_equal() {
            assert!(
                TerminalRendererMatrixCell {
                    display: ' ',
                    layer_of_value: Layer::base(),
                    background_color: None,
                    foreground_color: None,
                } == TerminalRendererMatrixCell {
                    display: ' ',
                    layer_of_value: Layer::base(),
                    background_color: None,
                    foreground_color: None,
                }
            );
        }

        #[test]
        fn cells_with_different_display_are_not_equal() {
            assert!(
                TerminalRendererMatrixCell {
                    display: ' ',
                    layer_of_value: Layer::base(),
                    background_color: None,
                    foreground_color: None,
                } != TerminalRendererMatrixCell {
                    display: '*',
                    layer_of_value: Layer::base(),
                    background_color: None,
                    foreground_color: None,
                }
            );
        }

        #[test]
        fn cells_with_different_layer_are_not_equal() {
            assert!(
                TerminalRendererMatrixCell {
                    display: ' ',
                    layer_of_value: Layer::furthest_foreground(),
                    background_color: None,
                    foreground_color: None,
                } != TerminalRendererMatrixCell {
                    display: ' ',
                    layer_of_value: Layer::base(),
                    background_color: None,
                    foreground_color: None,
                }
            );
        }

        #[test]
        fn cells_with_different_background_color_are_not_equal() {
            assert!(
                TerminalRendererMatrixCell {
                    display: ' ',
                    layer_of_value: Layer::base(),
                    background_color: None,
                    foreground_color: None,
                } != TerminalRendererMatrixCell {
                    display: ' ',
                    layer_of_value: Layer::base(),
                    background_color: Some(Rgb::white()),
                    foreground_color: None,
                }
            );
        }

        #[test]
        fn cells_with_different_foreground_color_are_not_equal() {
            assert!(
                TerminalRendererMatrixCell {
                    display: ' ',
                    layer_of_value: Layer::base(),
                    background_color: None,
                    foreground_color: None,
                } != TerminalRendererMatrixCell {
                    display: ' ',
                    layer_of_value: Layer::base(),
                    background_color: None,
                    foreground_color: Some(Rgb::magenta()),
                }
            );
        }
    }

    mod test_make_render_matrix {
        use super::*;

        mod without_camera_offset {
            use std::{cell::RefCell, rc::Rc};

            use crate::{Entity, QueryResult, StoredComponentList};

            use super::*;

            #[test]
            fn renderables_in_view_are_present() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(5, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (0, 3) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (5, 2) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_just_out_of_view_on_left_are_absent() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(5, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (0, 3) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (5, 2) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_just_out_of_view_on_right_edge_are_absent() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(5, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(10, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (0, 3) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (5, 2) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_just_out_of_view_on_top_edge_are_absent() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(5, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(3, -1),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (0, 3) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (5, 2) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_just_out_of_view_on_bottom_edge_are_absent() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(5, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(3, 10),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (0, 3) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (5, 2) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_on_screen_right_edge_are_present() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(9, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(5, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (9, 3) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (5, 2) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_on_screen_left_edge_are_present() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(5, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (0, 3) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (5, 2) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_on_screen_top_edge_are_present() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(5, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(1, 0),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (0, 3) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (5, 2) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (1, 0) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_on_screen_bottom_edge_are_present() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(5, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(1, 9),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (0, 3) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (5, 2) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (1, 9) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn higher_layer_renderable_appears_over_others_when_renderables_overlap() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::zero(),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::above(&Layer::base()),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(5, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '^',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(5, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (0, 3) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (5, 2) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::above(&Layer::base()),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }
        }

        mod with_camera_offset {
            use std::{cell::RefCell, rc::Rc};

            use crate::{Entity, QueryResult, StoredComponentList};

            use super::*;

            #[test]
            fn renderables_in_view_are_present() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::new(-6, 2),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(5, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'B',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (5, 0) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'B',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (6, 1) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_just_out_of_view_on_left_are_absent() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::new(-6, 2),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-7, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'B',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (5, 0) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'B',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (6, 1) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_just_out_of_view_on_right_edge_are_absent() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::new(-6, 2),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(4, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'B',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (5, 0) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'B',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (6, 1) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_just_out_of_view_on_top_edge_are_absent() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::new(-6, 2),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-3, 1),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'B',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (5, 0) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'B',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (6, 1) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_just_out_of_view_on_bottom_edge_are_absent() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::new(-6, 2),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-3, 12),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'B',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (5, 0) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'B',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (6, 1) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_on_screen_right_edge_are_present() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::new(-6, 2),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(3, 5),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'B',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (5, 0) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'B',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (6, 1) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (9, 3) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_on_screen_left_edge_are_present() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::new(-6, 2),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-6, 5),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'B',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (5, 0) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'B',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (6, 1) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (0, 3) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_on_screen_top_edge_are_present() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::new(-6, 2),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 2),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'B',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (5, 0) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'B',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (6, 1) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (6, 0) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }

            #[test]
            fn renderables_on_screen_bottom_edge_are_present() {
                let matrix = make_render_matrix(
                    &TerminalCamera {
                        field_of_view: Dimensions2d::new(10, 10),
                        is_main: true,
                    },
                    &TerminalTransform {
                        coords: IntCoords2d::new(-6, 2),
                    },
                    &QueryResultList::new(vec![
                        QueryResult::new(
                            Entity(0),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: '*',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(0, 3),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(1),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'A',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 11),
                                }))),
                            ]),
                        ),
                        QueryResult::new(
                            Entity(2),
                            StoredComponentList::new(vec![
                                Rc::new(RefCell::new(Box::new(TerminalRenderer {
                                    display: 'B',
                                    layer: Layer::base(),
                                    foreground_color: None,
                                    background_color: None,
                                }))),
                                Rc::new(RefCell::new(Box::new(TerminalTransform {
                                    coords: IntCoords2d::new(-1, 2),
                                }))),
                            ]),
                        ),
                    ]),
                    &TerminalRendererOptions {
                        screen_resolution: Dimensions2d::new(10, 10),
                        include_default_camera: true,
                        default_foreground_color: None,
                        default_background_color: None,
                    },
                );

                for cell in &*matrix {
                    match cell.location().values() {
                        (5, 0) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'B',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (6, 1) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: '*',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        (5, 9) => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: 'A',
                                    layer_of_value: Layer::base(),
                                }
                            );
                        }
                        _ => {
                            assert_eq!(
                                *cell.data(),
                                TerminalRendererMatrixCell {
                                    background_color: None,
                                    foreground_color: None,
                                    display: ' ',
                                    layer_of_value: Layer::furthest_background(),
                                }
                            );
                        }
                    }
                }
            }
        }
    }
}
