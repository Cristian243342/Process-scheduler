use core::module_path;
use function_name::named;
use processor::Processor;

use super::{run, scheduler};

#[test]
#[named]
pub fn send_receive() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                process.wait(1);
                for _ in 0..5 {
                    process.exec();
                }
            },
            0,
        );
        for _ in 0..5 {
            process.exec();
        }
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
pub fn workers() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                process.wait(1);
            },
            0,
        );
        process.fork(
            |process| {
                process.wait(1);
            },
            0,
        );
        process.fork(
            |process| {
                process.wait(2);
            },
            0,
        );
        for _ in 0..10 {
            process.exec();
        }
        process.signal(1);
        process.signal(2);
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
pub fn senders() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                process.wait(1);
                process.signal(2);
            },
            0,
        );
        process.fork(
            |process| {
                process.wait(2);
                process.signal(3);
            },
            0,
        );
        process.fork(
            |process| {
                process.wait(3);
            },
            0,
        );
        process.fork(
            |process| {
                process.wait(3);
            },
            0,
        );
        for _ in 0..10 {
            process.exec();
        }
        process.signal(1);
        process.sleep(10);
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}
