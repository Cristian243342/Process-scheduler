use function_name::named;
use processor::Processor;

use super::{run, scheduler};

#[test]
#[named]
pub fn wait() {
    let logs = Processor::run(scheduler(), |process| {
        for _ in 0..5 {
            process.exec();
        }
        process.wait(1);
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}

#[test]
#[named]
pub fn signal_before_wait() {
    let logs = Processor::run(scheduler(), |process| {
        for _ in 0..5 {
            process.exec();
        }
        process.signal(1);
        process.wait(1);
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}

#[test]
#[named]
pub fn wait_2() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                for _ in 0..5 {
                    process.exec();
                }
                process.wait(2);
            },
            0,
        );
        process.sleep(10);
        process.wait(1);
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
pub fn signal_before_wait_2() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                for _ in 0..5 {
                    process.exec();
                }
                process.wait(2);
            },
            0,
        );
        process.signal(2);
        process.wait(2);
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
pub fn wait_3() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                for _ in 0..5 {
                    process.exec();
                }
                process.wait(1);
            },
            0,
        );
        process.fork(
            |process| {
                for _ in 0..5 {
                    process.exec();
                }
                process.wait(1);
            },
            0,
        );
        process.fork(
            |process| {
                for _ in 0..5 {
                    process.exec();
                }
                process.wait(2);
            },
            0,
        );
        process.sleep(10);
        process.signal(1);
        process.wait(0);
        process.sleep(10);
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}
