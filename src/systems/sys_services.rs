use crate::{
    GameCommand, Input, Priority, Query, System, SystemsGenerator, Time, EVENT_BEFORE_UPDATE,
    EVENT_INIT, EVENT_AFTER_UPDATE,
};

pub struct ServicesSystemsGenerator {}
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
                EVENT_AFTER_UPDATE,
                System::new_with_priority(
                    Priority::lowest(),
                    vec![Query::new().has::<Time>(), Query::new().has::<Input>()],
                    |results, _| {
                        if let [time_results, input_results, ..] = &results[..] {
                            let mut time = time_results.get_only_mut::<Time>();
                            let mut input = input_results.get_only_mut::<Input>();

                            time.update();
                            input.update();
                        }
                    },
                ),
            ),
        ]
    }
}
