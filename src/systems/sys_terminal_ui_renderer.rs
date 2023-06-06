use std::{collections::HashMap, rc::Rc};

use crate::{
    Alignment, Component, GameCommand, GameCommandsArg, IntCoords2d, Layer, Query, QueryResultList,
    System, SystemsGenerator, TerminalRenderer, TerminalRendererOptions, TerminalRendererState,
    TerminalTextCharacter, TerminalTransform, Text, UiAnchor, EVENT_UPDATE,
};

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
                    Query::new().has::<TerminalRendererState>(),
                    Query::new()
                        .has::<TerminalTextCharacter>()
                        .has::<TerminalRenderer>(),
                ],
                |results, commands| {
                    if let [text_query, state_query, drawn_text_query, ..] = &results[..] {
                        let renderer_state = state_query
                            .get(0)
                            .expect(&format!("{} is available.", TerminalRendererState::name()))
                            .components()
                            .get::<TerminalRendererState>();

                        let anchor_positions = get_anchor_positions(&renderer_state.options);

                        wipe_existing_text(drawn_text_query, Rc::clone(&commands));

                        for text_result in text_query {
                            let text = text_result.components().get::<Text>();

                            let (anchor_x, anchor_y) = anchor_positions
                                .get(&text.anchor)
                                .expect("The anchor position can be determined.")
                                .values();

                            let chars = text.value.chars().collect::<Vec<char>>();

                            let justification_offset = match text.justification {
                                Alignment::Left => IntCoords2d::zero(),
                                Alignment::Middle => {
                                    IntCoords2d::new(-((chars.len() / 2) as i64), 0)
                                }
                                Alignment::Right => IntCoords2d::new(-(chars.len() as i64), 0),
                            };

                            let starting_position = IntCoords2d::new(anchor_x, anchor_y)
                                + justification_offset
                                + text.offset;

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
                },
            ),
        )]
    }
}

fn wipe_existing_text(text_character_query_results: &QueryResultList, commands: GameCommandsArg) {
    for text_character in text_character_query_results {
        commands
            .borrow_mut()
            .issue(GameCommand::DestroyEntity(*text_character.entity()));
    }
}

fn get_anchor_positions(options: &TerminalRendererOptions) -> HashMap<UiAnchor, IntCoords2d> {
    let (zero_indexed_width, zero_indexed_height) = (
        options.screen_resolution.width() as i64 - 1,
        options.screen_resolution.height() as i64 - 1,
    );

    HashMap::from([
        (UiAnchor::TopLeft, IntCoords2d::new(0, 0)),
        (
            UiAnchor::MiddleTop,
            IntCoords2d::new(zero_indexed_width / 2, 0),
        ),
        (UiAnchor::TopRight, IntCoords2d::new(zero_indexed_width, 0)),
        (
            UiAnchor::MiddleRight,
            IntCoords2d::new(zero_indexed_width, zero_indexed_height / 2),
        ),
        (
            UiAnchor::BottomRight,
            IntCoords2d::new(zero_indexed_width, zero_indexed_height),
        ),
        (
            UiAnchor::MiddleBottom,
            IntCoords2d::new(zero_indexed_width / 2, zero_indexed_height),
        ),
        (
            UiAnchor::BottomLeft,
            IntCoords2d::new(0, zero_indexed_height),
        ),
        (
            UiAnchor::MiddleLeft,
            IntCoords2d::new(0, zero_indexed_height),
        ),
    ])
}
