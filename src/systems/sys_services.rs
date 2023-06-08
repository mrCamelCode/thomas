use crate::{
    GameCommand, Input, Priority, Query, System, SystemsGenerator, Time, EVENT_AFTER_UPDATE,
    EVENT_BEFORE_UPDATE, EVENT_INIT,
};

pub(crate) struct ServicesSystemsGenerator {}
impl ServicesSystemsGenerator {
    pub fn new() -> Self {
        Self {}
    }
}
impl SystemsGenerator for ServicesSystemsGenerator {
    fn generate(&self) -> Vec<(&'static str, System)> {
        vec![
            (
                EVENT_INIT,
                System::new(vec![], |_, commands| {
                    commands.borrow_mut().issue(GameCommand::AddEntity(vec![
                        Box::new(Time::new()),
                        Box::new(Input::new()),
                    ]));
                }),
            ),
            (
                EVENT_BEFORE_UPDATE,
                System::new_with_priority(
                    Priority::highest(),
                    vec![Query::new().has::<Input>()],
                    |results, _| {
                        if let [input_results, ..] = &results[..] {
                            input_results.get_only_mut::<Input>().update();
                        }
                    },
                ),
            ),
            (
                EVENT_AFTER_UPDATE,
                System::new_with_priority(
                    Priority::lowest(),
                    vec![Query::new().has::<Time>()],
                    |results, _| {
                        if let [time_results, ..] = &results[..] {
                            time_results.get_only_mut::<Time>().update();
                        }
                    },
                ),
            ),
        ]
    }
}
