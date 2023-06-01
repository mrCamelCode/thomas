use std::collections::HashMap;

use crate::{
    Component, GameCommand, IntCoords2d, Layer, Query, QueryResultList, System, SystemExtraArgs,
    TerminalRenderer, TerminalRendererOptions, TerminalRendererState, TerminalTextCharacter,
    TerminalTransform, Text, UiAnchor,
};

pub struct TerminalUiRendererSystem {
    update_system: System,
}
impl TerminalUiRendererSystem {
    pub fn new() -> Self {
        Self {
            update_system: System::new(
                vec![
                    Query::new().has::<Text>(),
                    Query::new().has::<TerminalRendererState>(),
                    Query::new()
                        .has::<TerminalTextCharacter>()
                        .has::<TerminalRenderer>(),
                ],
                |results, util| {
                    if let [text_query, state_query, drawn_text_query, ..] = &results[..] {
                        let renderer_state = state_query
                            .get(0)
                            .expect(&format!("{} is available.", TerminalRendererState::name()))
                            .components()
                            .get::<TerminalRendererState>();

                        let anchor_positions = get_anchor_positions(&renderer_state.options);

                        wipe_existing_text(drawn_text_query, util);

                        for text_result in text_query {
                            let text = text_result.components().get::<Text>();

                            let (anchor_x, anchor_y) = anchor_positions
                                .get(&text.anchor)
                                .expect("The anchor position can be determined.")
                                .values();

                            // TODO: Look at the justification and offset to determine the more accurate position of the text.
                            let starting_position = IntCoords2d::new(anchor_x, anchor_y);

                            let mut offset = IntCoords2d::zero();
                            for character in text.value.chars() {
                                util.commands().issue(GameCommand::AddEntity(vec![
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
        }
    }

    pub fn extract_systems(self) -> System {
        self.update_system
    }
}

fn wipe_existing_text(text_character_query_results: &QueryResultList, util: &SystemExtraArgs) {
    for text_character in text_character_query_results {
        util.commands()
            .issue(GameCommand::DestroyEntity(*text_character.entity()));
    }
}

fn get_anchor_positions(options: &TerminalRendererOptions) -> HashMap<UiAnchor, IntCoords2d> {
    let (width, height) = (
        options.screen_resolution.width() as i64,
        options.screen_resolution.height() as i64,
    );

    HashMap::from([
        (UiAnchor::TopLeft, IntCoords2d::new(0, 0)),
        (UiAnchor::MiddleTop, IntCoords2d::new(width / 2, 0)),
        (UiAnchor::TopRight, IntCoords2d::new(width, 0)),
        (UiAnchor::MiddleRight, IntCoords2d::new(width, height / 2)),
        (UiAnchor::BottomRight, IntCoords2d::new(width, height)),
        (UiAnchor::MiddleBottom, IntCoords2d::new(width / 2, height)),
        (UiAnchor::BottomLeft, IntCoords2d::new(0, height)),
        (UiAnchor::MiddleLeft, IntCoords2d::new(0, height)),
    ])
}
