use std::{
    io::stdout,
    io::Write,
    ops::{Deref, DerefMut},
};

use crossterm::{
    cursor, execute,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
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

                                state.prev_render = Some(draw(
                                    &*main_camera,
                                    &*main_camera_transform,
                                    &renderables_query,
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
                            "The terminal may be in a bad state. It's recommended to start a new terminal instance if you want to use the terminal.";

                            if let Err(e) = execute!(
                                stdout(),
                                SetSize(
                                    state.initial_terminal_size.0,
                                    state.initial_terminal_size.1
                                ),
                                cursor::Show,
                                cursor::RestorePosition,
                                Clear(ClearType::All),
                                ResetColor
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
    let render_matrix = make_render_matrix(
        main_camera,
        main_camera_transform,
        renderables_query_result,
        renderer_options,
    );

    for col in 0..render_matrix.dimensions().width() {
        for row in 0..render_matrix.dimensions().height() {
            let cell = render_matrix.get(col, row).unwrap();
            let prev_cell = if let Some(prev_render) = previous_render {
                prev_render.get(col, row)
            } else {
                None
            };

            if prev_cell.is_none() || cell.data() != prev_cell.unwrap().data() {
                if let Err(e) = execute!(stdout(), cursor::MoveTo(col as u16, row as u16)) {
                    panic!(
                        "Error occurred while trying to move the cursor to position ({}, {}): {e}",
                        col as u16, row as u16
                    );
                }

                if let Err(e) = execute!(
                    stdout(),
                    SetForegroundColor(get_crossterm_color(
                        &cell.data().foreground_color,
                        &renderer_options.default_foreground_color
                    )),
                    SetBackgroundColor(get_crossterm_color(
                        &cell.data().background_color,
                        &renderer_options.default_background_color
                    )),
                ) {
                    panic!("Error occurred while trying to set the cell's draw color at ({col}, {row}): {e}");
                }

                if let Err(e) = write!(stdout(), "{}", cell.data().display) {
                    panic!("Error occurred while trying to update the displayed character at cell ({col}, {row}): {e}");
                }
            }
        }
    }

    render_matrix
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
    let mut render_matrix = TerminalRendererMatrix::new(renderer_options.screen_resolution);

    for result in renderables_query_result {
        let renderable_transform = result.components().get::<TerminalTransform>();
        let renderable_screen_position = renderable_transform.coords - main_camera_transform.coords;

        let TerminalRenderer {
            display,
            layer,
            foreground_color,
            background_color,
        } = &*result.components().get::<TerminalRenderer>();

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
                            // display: make_colored_character(
                            //     *display,
                            //     foreground_color,
                            //     background_color,
                            // ),
                            layer_of_value: layer.clone(),
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
    pub include_default_camera: bool,
    pub default_foreground_color: Option<Rgb>,
    pub default_background_color: Option<Rgb>,
}

#[derive(Debug)]
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

#[derive(Debug, PartialEq, Eq)]
struct TerminalRendererMatrixCell {
    display: char,
    layer_of_value: Layer,
    foreground_color: Option<Rgb>,
    background_color: Option<Rgb>,
}
impl TerminalRendererMatrixCell {
    fn default() -> Self {
        Self {
            display: ' ',
            layer_of_value: Layer::furthest_background(),
            foreground_color: None,
            background_color: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            fn renderables_out_of_view_are_absent() {
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
        }
    }
}
