use function_name::named;
use processor::Processor;

use super::{run, scheduler};

#[test]
#[named]
pub fn exec() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                for _ in 0..5 {
                    process.exec();
                }
            },
            0,
        );
        process.exec();
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}

#[test]
#[named]
pub fn sleep() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                process.sleep(5);
            },
            0,
        );
        process.exec();
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}

#[test]
#[named]
pub fn wait() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                process.wait(1);
            },
            0,
        );
        process.exec();
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}
