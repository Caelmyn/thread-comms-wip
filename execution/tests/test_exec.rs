use execution::{Exec, Runtime, RunState};
use execution::commands;

#[test]
fn exec() {
    fn count(state: &RunState, count_to: u32) {
        let mut counter = 0u32;

        while state.is_running() && counter < count_to {
            counter += 1;
        }
    }

    let rt = Runtime::new(count);
    let ex = Exec::build(rt).spawn_pipe();

    ex.send(commands::Spawn("toto", 50_000_000u32)).unwrap();
}
