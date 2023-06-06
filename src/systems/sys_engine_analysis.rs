use std::collections::VecDeque;

use crate::{
    EngineStats, GameCommand, Query, System, SystemsGenerator, Time, Timer, EVENT_AFTER_UPDATE,
    EVENT_BEFORE_UPDATE, EVENT_INIT,
};

const NUM_POLLED_SECONDS_FOR_FRAMERATE: u8 = 10;

pub struct EngineAnalysisSystemsGenerator {}
impl EngineAnalysisSystemsGenerator {
    pub fn new() -> Self {
        Self {}
    }
}
impl SystemsGenerator for EngineAnalysisSystemsGenerator {
    fn generate(&self) -> Vec<(&'static str, System)> {
        vec![
            (
                EVENT_INIT,
                System::new(vec![], |_, commands| {
                    commands
                        .borrow_mut()
                        .issue(GameCommand::AddEntity(vec![Box::new(EngineStats {
                            fps: 0,
                            frame_timer: Timer::new(),
                            frame_counter: 0,
                            frame_counts: VecDeque::new(),
                        })]));
                }),
            ),
            (
                EVENT_BEFORE_UPDATE,
                System::new(vec![Query::new().has::<EngineStats>()], |results, _| {
                    if let [engine_stats_results, ..] = &results[..] {
                        let mut engine_stats = engine_stats_results.get_only_mut::<EngineStats>();

                        if !engine_stats.frame_timer.is_running() {
                            engine_stats.frame_timer.start();
                        }

                        if engine_stats.frame_timer.elapsed_millis() >= 1000 {
                            engine_stats.frame_timer.restart();

                            while engine_stats.frame_counts.len()
                                >= NUM_POLLED_SECONDS_FOR_FRAMERATE as usize
                            {
                                engine_stats.frame_counts.pop_front();
                            }

                            let count = engine_stats.frame_counter;
                            engine_stats.frame_counts.push_back(count);

                            engine_stats.frame_counter = 0;
                        }

                        engine_stats.frame_counter += 1;
                        engine_stats.fps = if engine_stats.frame_counts.len() > 0 {
                            engine_stats.frame_counts.iter().sum::<u64>()
                                / engine_stats.frame_counts.len() as u64
                        } else {
                            0
                        };
                    }
                }),
            ),
        ]
    }
}
