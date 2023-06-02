use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use crate::{GameCommandQueue, Input, Query, QueryResultList, Time, Priority};

pub type OperatorFn = dyn Fn(Vec<QueryResultList>, &SystemExtraArgs) -> ();
pub struct System {
    queries: Vec<Query>,
    operator: Box<OperatorFn>,
    priority: Priority,
}
impl System {
    pub fn new(
        queries: Vec<Query>,
        operator: impl Fn(Vec<QueryResultList>, &SystemExtraArgs) -> () + 'static,
    ) -> Self {
        Self {
            queries,
            operator: Box::new(operator),
            priority: Priority::default(),
        }
    }

    pub fn new_with_priority(
        priority: Priority,
        queries: Vec<Query>,
        operator: impl Fn(Vec<QueryResultList>, &SystemExtraArgs) -> () + 'static,
    ) -> Self {
        Self {
            queries,
            operator: Box::new(operator),
            priority
        }
    }

    pub(crate) fn queries(&self) -> &Vec<Query> {
        &self.queries
    }

    pub(crate) fn operator(&self) -> &OperatorFn {
        &self.operator
    }

    pub(crate) fn priority(&self) -> &Priority {
        &self.priority
    }
}

const ARGS_KEY_GAME_COMMAND_QUEUE: &str = "gcqueue";
const ARGS_KEY_INPUT: &str = "input";
const ARGS_KEY_TIME: &str = "time";

pub type CustomExtraArgs = Vec<(&'static str, Box<dyn Any>)>;

pub struct SystemExtraArgs {
    args: HashMap<&'static str, Box<dyn Any>>,
}
impl SystemExtraArgs {
    pub(crate) fn new(
        game_command_queue: Rc<RefCell<GameCommandQueue>>,
        input: Rc<RefCell<Input>>,
        time: Rc<RefCell<Time>>,
        custom_pairs: CustomExtraArgs,
    ) -> Self {
        let mut map = HashMap::from([
            (
                ARGS_KEY_GAME_COMMAND_QUEUE,
                Box::new(game_command_queue) as Box<dyn Any>,
            ),
            (ARGS_KEY_INPUT, Box::new(input)),
            (ARGS_KEY_TIME, Box::new(time)),
        ]);

        for (key, value) in custom_pairs {
            map.insert(key, value);
        }

        Self { args: map }
    }

    pub fn commands(&self) -> RefMut<GameCommandQueue> {
        self.args
            .get(ARGS_KEY_GAME_COMMAND_QUEUE)
            .expect("The GameCommandQueue is available.")
            .downcast_ref::<Rc<RefCell<GameCommandQueue>>>()
            .unwrap()
            .borrow_mut()
    }

    pub fn time(&self) -> Ref<Time> {
        Ref::map(
            self.args
                .get(ARGS_KEY_TIME)
                .expect("The Time service is available.")
                .downcast_ref::<Rc<RefCell<Time>>>()
                .unwrap()
                .borrow(),
            |time| time,
        )
    }

    pub fn input(&self) -> Ref<Input> {
        Ref::map(
            self.args
                .get(ARGS_KEY_INPUT)
                .expect("The Input service is available.")
                .downcast_ref::<Rc<RefCell<Input>>>()
                .unwrap()
                .borrow(),
            |input| input,
        )
    }

    pub fn try_get<T: 'static>(&self, key: &str) -> Option<&T> {
        if let Some(value) = self.args.get(key) {
            if let Some(cast_value) = value.downcast_ref::<T>() {
                return Some(cast_value);
            }
        }

        None
    }

    pub fn get<T: 'static>(&self, key: &str) -> &T {
        self.args
            .get(key)
            .expect(&format!("Key {key} is available in SystemExtraArgs."))
            .downcast_ref::<T>()
            .expect("Ref could be downcast to type T.")
    }
}

pub trait SystemsGenerator {
    fn generate(&self) -> Vec<(&'static str, System)>;
}
#[cfg(test)]
mod tests {
    use super::*;

    mod system_extra_args {
        use crate::GameCommand;

        use super::*;

        mod builtin_keys {
            use super::*;

            #[test]
            fn game_command_queue_can_be_retrieved() {
                let input = Rc::new(RefCell::new(Input::new()));
                let time = Rc::new(RefCell::new(Time::new()));

                let extra_args = SystemExtraArgs::new(
                    Rc::new(RefCell::new(GameCommandQueue::new())),
                    Rc::clone(&input),
                    Rc::clone(&time),
                    vec![],
                );

                extra_args.commands();
            }

            #[test]
            fn game_command_queue_can_be_mutated() {
                let input = Rc::new(RefCell::new(Input::new()));
                let time = Rc::new(RefCell::new(Time::new()));

                let extra_args = SystemExtraArgs::new(
                    Rc::new(RefCell::new(GameCommandQueue::new())),
                    Rc::clone(&input),
                    Rc::clone(&time),
                    vec![],
                );

                let mut commands = extra_args.commands();

                commands.issue(GameCommand::Quit);

                for command in &*commands {
                    match command {
                        GameCommand::Quit => {
                            assert!(true);
                        }
                        _ => panic!("Command was not GameCommand::Quit"),
                    }
                }
            }

            #[test]
            fn input_service_can_be_accessed() {
                let input = Rc::new(RefCell::new(Input::new()));
                let time = Rc::new(RefCell::new(Time::new()));

                let extra_args = SystemExtraArgs::new(
                    Rc::new(RefCell::new(GameCommandQueue::new())),
                    Rc::clone(&input),
                    Rc::clone(&time),
                    vec![],
                );

                let input = extra_args.input();

                assert!(!input.is_key_down(&device_query::Keycode::A));
            }

            #[test]
            fn time_service_can_be_accessed() {
                let input = Rc::new(RefCell::new(Input::new()));
                let time = Rc::new(RefCell::new(Time::new()));

                let extra_args = SystemExtraArgs::new(
                    Rc::new(RefCell::new(GameCommandQueue::new())),
                    Rc::clone(&input),
                    Rc::clone(&time),
                    vec![],
                );

                let time = extra_args.time();

                assert_eq!(!time.delta_time(), u128::MAX);
            }
        }

        mod custom_keys {
            use super::*;

            const KEY_CUSTOM: &str = "custom";

            #[test]
            fn can_get_data_off_custom_key() {
                let input = Rc::new(RefCell::new(Input::new()));
                let time = Rc::new(RefCell::new(Time::new()));

                let extra_args = SystemExtraArgs::new(
                    Rc::new(RefCell::new(GameCommandQueue::new())),
                    Rc::clone(&input),
                    Rc::clone(&time),
                    vec![(KEY_CUSTOM, Box::new(10 as i32))],
                );

                let data_option = extra_args.try_get::<i32>(KEY_CUSTOM);

                assert!(data_option.is_some());
                assert_eq!(*data_option.unwrap(), 10);
            }

            #[test]
            fn can_get_complex_data_off_custom_key() {
                #[derive(PartialEq, Eq, Debug)]
                struct Complex {
                    prop1: i32,
                    prop2: String,
                }

                let input = Rc::new(RefCell::new(Input::new()));
                let time = Rc::new(RefCell::new(Time::new()));

                let extra_args = SystemExtraArgs::new(
                    Rc::new(RefCell::new(GameCommandQueue::new())),
                    Rc::clone(&input),
                    Rc::clone(&time),
                    vec![(
                        KEY_CUSTOM,
                        Box::new(Complex {
                            prop1: 150,
                            prop2: String::from("Hello world!"),
                        }),
                    )],
                );

                let data = extra_args.get::<Complex>(KEY_CUSTOM);

                assert_eq!(
                    *data,
                    Complex {
                        prop1: 150,
                        prop2: String::from("Hello world!"),
                    }
                );
            }
        }
    }
}
