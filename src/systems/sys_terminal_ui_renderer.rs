use std::{collections::HashMap, rc::Rc};

use crate::{
    Alignment, GameCommand, GameCommandsArg, IntCoords2d, Layer, Query, QueryResultList, System,
    SystemsGenerator, TerminalCamera, TerminalRenderer, TerminalTextCharacter, TerminalTransform,
    Text, UiAnchor, EVENT_UPDATE,
};

/// A generator responsible for setting up and performing UI rendering in a terminal game. This must be added to the game
/// to enable UI rendering.
///
/// UI is positioned according to the screen space of the main camera. Even if the camera moves, UI elements will always
/// remain in the same location relative to the camera.
pub struct TerminalUiRendererSystemsGenerator {}
impl TerminalUiRendererSystemsGenerator {
    pub fn new() -> Self {
        Self {}
    }
}
impl SystemsGenerator for TerminalUiRendererSystemsGenerator {
    fn generate(&self) -> Vec<(&'static str, System)> {
        vec![(
            EVENT_UPDATE,
            System::new(
                vec![
                    Query::new().has::<Text>(),
                    Query::new()
                        .has::<TerminalTextCharacter>()
                        .has::<TerminalRenderer>(),
                    Query::new()
                        .has_where::<TerminalCamera>(|cam| cam.is_main)
                        .has::<TerminalTransform>(),
                ],
                update_ui,
            ),
        )]
    }
}

fn update_ui(results: Vec<QueryResultList>, commands: GameCommandsArg) {
    if let [text_results, drawn_text_results, main_cam_results, ..] = &results[..] {
        let main_cam = main_cam_results.get_only::<TerminalCamera>();
        let main_cam_transform = main_cam_results.get_only::<TerminalTransform>();

        let anchor_positions = get_anchor_positions(&main_cam, &main_cam_transform);

        wipe_existing_text(drawn_text_results, Rc::clone(&commands));

        for text_result in text_results {
            let text = text_result.components().get::<Text>();

            let (anchor_x, anchor_y) = anchor_positions
                .get(&text.anchor)
                .expect("The anchor position can be determined.")
                .values();

            let chars = text.value.chars().collect::<Vec<char>>();

            let justification_offset = match text.justification {
                Alignment::Left => IntCoords2d::zero(),
                Alignment::Middle => IntCoords2d::new(-((chars.len() / 2) as i64), 0),
                Alignment::Right => IntCoords2d::new(-(chars.len() as i64), 0),
            };

            let starting_position =
                IntCoords2d::new(anchor_x, anchor_y) + justification_offset + text.offset;

            let mut offset = IntCoords2d::zero();
            for character in chars {
                commands.borrow_mut().issue(GameCommand::AddEntity(vec![
                    Box::new(TerminalTextCharacter {}),
                    Box::new(TerminalRenderer {
                        display: character,
                        layer: Layer::below(&Layer::furthest_foreground()),
                    }),
                    Box::new(TerminalTransform {
                        coords: starting_position + offset,
                    }),
                ]));

                offset += IntCoords2d::right();
            }
        }
    }
}

fn wipe_existing_text(text_character_query_results: &QueryResultList, commands: GameCommandsArg) {
    for text_character in text_character_query_results {
        commands
            .borrow_mut()
            .issue(GameCommand::DestroyEntity(*text_character.entity()));
    }
}

fn get_anchor_positions(
    main_camera: &TerminalCamera,
    main_camera_transform: &TerminalTransform,
) -> HashMap<UiAnchor, IntCoords2d> {
    let (zero_indexed_width, zero_indexed_height) = (
        main_camera.field_of_view.width() as i64 - 1,
        main_camera.field_of_view.height() as i64 - 1,
    );

    let base_coords = main_camera_transform.coords;

    HashMap::from([
        (UiAnchor::TopLeft, base_coords),
        (
            UiAnchor::MiddleTop,
            base_coords + IntCoords2d::new(zero_indexed_width / 2, 0),
        ),
        (
            UiAnchor::TopRight,
            base_coords + IntCoords2d::new(zero_indexed_width, 0),
        ),
        (
            UiAnchor::MiddleRight,
            base_coords + IntCoords2d::new(zero_indexed_width, zero_indexed_height / 2),
        ),
        (
            UiAnchor::BottomRight,
            base_coords + IntCoords2d::new(zero_indexed_width, zero_indexed_height),
        ),
        (
            UiAnchor::MiddleBottom,
            base_coords + IntCoords2d::new(zero_indexed_width / 2, zero_indexed_height),
        ),
        (
            UiAnchor::BottomLeft,
            base_coords + IntCoords2d::new(0, zero_indexed_height),
        ),
        (
            UiAnchor::MiddleLeft,
            base_coords + IntCoords2d::new(0, zero_indexed_height / 2),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_update_ui {
        use std::cell::RefCell;

        use crate::{Dimensions2d, Entity, QueryResult, StoredComponentList};

        use super::*;

        mod with_camera_at_origin {
            use crate::{Component, GameCommandQueue};

            use super::*;

            fn make_basic_results() -> Vec<QueryResultList> {
                let results = vec![
                    QueryResultList::new(vec![]),
                    QueryResultList::new(vec![]),
                    QueryResultList::new(vec![QueryResult::new(
                        Entity(1),
                        StoredComponentList::new(vec![
                            Rc::new(RefCell::new(Box::new(TerminalCamera {
                                field_of_view: Dimensions2d::new(10, 10),
                                is_main: true,
                            }))),
                            Rc::new(RefCell::new(Box::new(TerminalTransform {
                                coords: IntCoords2d::zero(),
                            }))),
                        ]),
                    )]),
                ];

                results
            }

            #[test]
            fn pos_is_correct_for_top_left() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::TopLeft,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(0, 0)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_middle_top() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::MiddleTop,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(4, 0)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_top_right() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::TopRight,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(9, 0)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_middle_right() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::MiddleRight,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(9, 4)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_bottom_right() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::BottomRight,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(9, 9)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_middle_bottom() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::MiddleBottom,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(4, 9)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_bottom_left() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::BottomLeft,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(0, 9)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_middle_left() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::MiddleLeft,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(0, 4)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }
        }

        mod with_camera_offset_from_origin {
            use super::*;
            use crate::{Component, GameCommandQueue};

            fn make_basic_results() -> Vec<QueryResultList> {
                let results = vec![
                    QueryResultList::new(vec![]),
                    QueryResultList::new(vec![]),
                    QueryResultList::new(vec![QueryResult::new(
                        Entity(1),
                        StoredComponentList::new(vec![
                            Rc::new(RefCell::new(Box::new(TerminalCamera {
                                field_of_view: Dimensions2d::new(5, 5),
                                is_main: true,
                            }))),
                            Rc::new(RefCell::new(Box::new(TerminalTransform {
                                coords: IntCoords2d::new(-3, 2),
                            }))),
                        ]),
                    )]),
                ];

                results
            }

            #[test]
            fn pos_is_correct_for_top_left() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::TopLeft,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(-3, 2)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_middle_top() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::MiddleTop,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(-1, 2)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_top_right() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::TopRight,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(1, 2)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_middle_right() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::MiddleRight,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(1, 4)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_bottom_right() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::BottomRight,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(1, 6)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_middle_bottom() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::MiddleBottom,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(-1, 6)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_bottom_left() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::BottomLeft,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(-3, 6)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }

            #[test]
            fn pos_is_correct_for_middle_left() {
                let mut results = make_basic_results();

                results[0].push(QueryResult::new(
                    Entity(0),
                    StoredComponentList::new(vec![Rc::new(RefCell::new(Box::new(Text {
                        value: String::from("T"),
                        anchor: UiAnchor::MiddleLeft,
                        justification: Alignment::Left,
                        offset: IntCoords2d::zero(),
                    })))]),
                ));

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_ui(results, Rc::clone(&commands));

                assert!(commands
                    .borrow()
                    .queue()
                    .iter()
                    .find(|c| match c {
                        GameCommand::AddEntity(comps) => {
                            comps
                                .iter()
                                .find(|comp| {
                                    comp.component_name() == TerminalTransform::name()
                                        && (TerminalTransform::cast(&***comp)).unwrap().coords
                                            == IntCoords2d::new(-3, 4)
                                })
                                .is_some()
                        }
                        _ => {
                            false
                        }
                    })
                    .is_some());
            }
        }
    }
}
