use std::{collections::HashMap, rc::Rc};

use crate::{
    Alignment, GameCommand, GameCommandsArg, IntCoords2d, Layer, Query, QueryResultList, Rgb,
    System, SystemsGenerator, TerminalCamera, TerminalRenderer, TerminalTextCharacter,
    TerminalTransform, Text, UiAnchor, WorldText, EVENT_UPDATE,
};

/// A generator responsible for setting up and performing UI rendering in a terminal game. This systems generator is
/// included for you, you don't need to include it.
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
                    Query::new().has::<WorldText>().has::<TerminalTransform>(),
                    Query::new()
                        .has::<TerminalTextCharacter>()
                        .has::<TerminalRenderer>(),
                    Query::new()
                        .has_where::<TerminalCamera>(|cam| cam.is_main)
                        .has::<TerminalTransform>(),
                ],
                update_text_ui,
            ),
        )]
    }
}

fn update_text_ui(results: Vec<QueryResultList>, commands: GameCommandsArg) {
    if let [text_results, world_text_results, drawn_text_results, main_cam_results, ..] =
        &results[..]
    {
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

            let justification_offset = get_justification_offset(&text.justification, chars.len());

            let starting_position =
                IntCoords2d::new(anchor_x, anchor_y) + justification_offset + text.offset;

            add_text_entities(
                &chars,
                &starting_position,
                &text.foreground_color,
                &text.background_color,
                Rc::clone(&commands),
            );
        }

        for world_text_result in world_text_results {
            let world_text = world_text_result.components().get::<WorldText>();
            let world_text_transform = world_text_result.components().get::<TerminalTransform>();

            let chars = world_text.value.chars().collect::<Vec<char>>();

            let justification_offset =
                get_justification_offset(&world_text.justification, chars.len());

            let starting_position =
                world_text_transform.coords + justification_offset + world_text.offset;

            add_text_entities(
                &chars,
                &starting_position,
                &world_text.foreground_color,
                &world_text.background_color,
                Rc::clone(&commands),
            );
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

fn get_justification_offset(justification: &Alignment, text_length: usize) -> IntCoords2d {
    match justification {
        Alignment::Left => IntCoords2d::zero(),
        Alignment::Middle => IntCoords2d::new(-((text_length / 2) as i64), 0),
        Alignment::Right => IntCoords2d::new(-(text_length as i64), 0),
    }
}

fn add_text_entities(
    chars: &Vec<char>,
    starting_position: &IntCoords2d,
    foreground_color: &Option<Rgb>,
    background_color: &Option<Rgb>,
    commands: GameCommandsArg,
) {
    let mut offset = IntCoords2d::zero();
    for character in chars {
        commands.borrow_mut().issue(GameCommand::AddEntity(vec![
            Box::new(TerminalTextCharacter {}),
            Box::new(TerminalRenderer {
                display: *character,
                layer: Layer::below(&Layer::furthest_foreground()),
                foreground_color: *foreground_color,
                background_color: *background_color,
            }),
            Box::new(TerminalTransform {
                coords: *starting_position + offset,
            }),
        ]));

        offset += IntCoords2d::right();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_update_text_ui {
        use std::cell::RefCell;

        use crate::{Dimensions2d, Entity, QueryResult, StoredComponentList};

        use super::*;

        mod text {
            use super::*;

            mod with_camera_at_origin {
                use crate::{Component, GameCommandQueue};

                use super::*;

                fn make_basic_results() -> Vec<QueryResultList> {
                    let results = vec![
                        QueryResultList::new(vec![]),
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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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
                            foreground_color: None,
                            background_color: None,
                        })))]),
                    ));

                    let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                    update_text_ui(results, Rc::clone(&commands));

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

        mod world_text {
            use crate::{Component, GameCommandQueue};

            use super::*;

            #[test]
            fn text_goes_to_the_correct_position_with_origin_camera() {
                let results = vec![
                    QueryResultList::new(vec![]),
                    QueryResultList::new(vec![QueryResult::new(
                        Entity(10),
                        StoredComponentList::new(vec![
                            Rc::new(RefCell::new(Box::new(WorldText {
                                value: String::from("T"),
                                justification: Alignment::Left,
                                offset: IntCoords2d::zero(),
                                background_color: None,
                                foreground_color: None,
                            }))),
                            Rc::new(RefCell::new(Box::new(TerminalTransform {
                                coords: IntCoords2d::new(5, 3),
                            }))),
                        ]),
                    )]),
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

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_text_ui(results, Rc::clone(&commands));

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
                                            == IntCoords2d::new(5, 3)
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
            fn text_goes_to_the_correct_position_with_offset_camera() {
                let results = vec![
                    QueryResultList::new(vec![]),
                    QueryResultList::new(vec![QueryResult::new(
                        Entity(10),
                        StoredComponentList::new(vec![
                            Rc::new(RefCell::new(Box::new(WorldText {
                                value: String::from("T"),
                                justification: Alignment::Left,
                                offset: IntCoords2d::zero(),
                                background_color: None,
                                foreground_color: None,
                            }))),
                            Rc::new(RefCell::new(Box::new(TerminalTransform {
                                coords: IntCoords2d::new(5, 3),
                            }))),
                        ]),
                    )]),
                    QueryResultList::new(vec![]),
                    QueryResultList::new(vec![QueryResult::new(
                        Entity(1),
                        StoredComponentList::new(vec![
                            Rc::new(RefCell::new(Box::new(TerminalCamera {
                                field_of_view: Dimensions2d::new(10, 10),
                                is_main: true,
                            }))),
                            Rc::new(RefCell::new(Box::new(TerminalTransform {
                                coords: IntCoords2d::new(-5, 8),
                            }))),
                        ]),
                    )]),
                ];

                let commands = Rc::new(RefCell::new(GameCommandQueue::new()));

                update_text_ui(results, Rc::clone(&commands));

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
                                            == IntCoords2d::new(5, 3)
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
