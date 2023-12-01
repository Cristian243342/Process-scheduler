use core::module_path;
use function_name::named;
use processor::Processor;

use super::{run, scheduler};

#[test]
#[named]
pub fn single_process() {
    let logs = Processor::run(scheduler(), |process| {
        for _ in 0..5 {
            process.exec();
        }
    });
    println!("{:?}", logs);
    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}

#[test]
#[named]
pub fn fork_2() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                for _ in 0..5 {
                    process.exec();
                }
            },
            0,
        );
        for _ in 0..10 {
            process.exec();
        }
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}

#[test]
#[named]
pub fn fork_3() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                process.fork(
                    |process| {
                        for _ in 0..5 {
                            process.exec();
                        }
                    },
                    0,
                );
                for _ in 0..5 {
                    process.exec();
                }
            },
            0,
        );
        for _ in 0..10 {
            process.exec();
        }
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
        process.sleep(10);
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}

#[test]
#[named]
pub fn work_sleep() {
    let logs = Processor::run(scheduler(), |process| {
        for _ in 0..3 {
            process.exec();
        }
        process.sleep(10);
        for _ in 0..3 {
            process.exec();
        }
        process.sleep(10);
        for _ in 0..3 {
            process.exec();
        }
        process.sleep(10);
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}

#[test]
#[named]
pub fn fork_wait_signal() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                process.wait(1);
            },
            0,
        );
        process.sleep(10);
        process.signal(1);
        process.sleep(10);
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}

#[test]
#[named]
pub fn fork_wait_sleep_signal() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                process.wait(1);
            },
            0,
        );
        process.sleep(5);
        process.signal(1);
        process.sleep(10);
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}
