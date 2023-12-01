use core::module_path;
use function_name::named;
use processor::Processor;

use super::{run, scheduler};

#[test]
#[named]
pub fn single_worker() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                for _ in 0..20 {
                    process.exec();
                }
            },
            5,
        );
        for _ in 0..30 {
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
pub fn worker_io() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                for _ in 0..10 {
                    process.exec();
                }
                for _ in 0..5 {
                    process.sleep(1);
                    process.exec();
                    process.exec();
                }
            },
            3,
        );
        for _ in 0..50 {
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
pub fn worker_3() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                for _ in 0..10 {
                    process.exec();
                }
            },
            3,
        );
        process.fork(
            |process| {
                for _ in 0..20 {
                    process.sleep(1);
                    process.exec();
                    process.exec();
                }
            },
            5,
        );
        for _ in 0..50 {
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
pub fn worker_spawning() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                for _ in 0..20 {
                    process.exec();
                }
                process.fork(
                    |process| {
                        for _ in 0..20 {
                            process.exec();
                        }
                    },
                    5,
                );
            },
            5,
        );
        for _ in 0..50 {
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
pub fn sleeper() {
    let logs = Processor::run(scheduler(), |process| {
        process.fork(
            |process| {
                process.fork(
                    |process| {
                        for _ in 0..20 {
                            process.exec();
                        }
                    },
                    5,
                );
                for _ in 0..20 {
                    process.exec();
                }
                process.fork(
                    |process| {
                        for _ in 0..20 {
                            process.exec();
                        }
                    },
                    5,
                );      
            },
            5,
        );
        process.sleep(110);
        for _ in 0..50 {
            process.exec();
        }
    });

    run(
        module_path!().split("::").last().unwrap(),
        function_name!(),
        &logs,
    );
}
