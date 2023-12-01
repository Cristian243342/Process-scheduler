#[cfg(feature = "cfs")]
use scheduler::cfs;
#[cfg(feature = "priority-queue")]
use scheduler::priority_queue;
#[cfg(not(any(feature = "priority-queue", feature = "cfs")))]
use scheduler::round_robin;
use scheduler::Scheduler;

use std::env;
use std::fs;

use processor::format_logs;
use processor::Log;
use std::num::NonZeroUsize;

mod deadlock;
mod panic;
mod simple;
mod wait_and_signal;
mod workers;

fn write_logs(folder: &str, name: &str, logs: &str) {
    let (timeslice, remaining, cpu_slices) = arguments();
    fs::create_dir_all(format!("../outputs/{SCHEDULER}/{folder}")).unwrap();
    fs::write(
        format!(
            "../outputs/{SCHEDULER}/{folder}/{name}___{timeslice}_{remaining}_{cpu_slices}.log"
        ),
        logs,
    )
    .unwrap();
}

fn read_logs(folder: &str, name: &str) -> String {
    let (timeslice, remaining, cpu_slices) = arguments();
    fs::read_to_string(format!(
        "../outputs/{SCHEDULER}/{folder}/{name}___{timeslice}_{remaining}_{cpu_slices}.log"
    ))
    .unwrap()
}

fn run(folder: &str, name: &str, logs: &[Log]) {
    let output = format_logs(&logs);

    if env::var("WRITE_OUTPUT").is_ok() {
        write_logs(folder, name, &output);
    } else {
        let reference = read_logs(folder, name);

        println!("\nleft = Correct Output\nright = Your Output\n");
        use pretty_assertions::assert_eq;
        assert_eq!(reference, output);
    }
}

fn arguments() -> (usize, usize, usize) {
    let timeslice = env::var("TIMESLICE")
        .unwrap_or("3".to_string())
        .parse::<usize>()
        .unwrap();
    let remaining = env::var("REMAINING")
        .unwrap_or("1".to_string())
        .parse::<usize>()
        .unwrap();
    let cpu_slices = env::var("CPU_SLICES")
        .unwrap_or("10".to_string())
        .parse::<usize>()
        .unwrap();
    (timeslice, remaining, cpu_slices)
}

#[cfg(feature = "round-robin")]
static SCHEDULER: &str = "round-robin";
#[cfg(feature = "round-robin")]
fn scheduler() -> impl Scheduler {
    let (timeslice, remaining, cpu_slices) = arguments();

    println!("Timeslice {timeslice}\nRemaining {remaining}\nCPU slices: {cpu_slices}");
    round_robin(NonZeroUsize::new(timeslice).unwrap(), remaining)
}

#[cfg(feature = "priority-queue")]
static SCHEDULER: &str = "priority-queue";
#[cfg(feature = "priority-queue")]
fn scheduler() -> impl Scheduler {
    let (timeslice, remaining, cpu_slices) = arguments();

    println!("Timeslice {timeslice}\nRemaining {remaining}\nCPU slices: {cpu_slices}");

    priority_queue(NonZeroUsize::new(timeslice).unwrap(), remaining)
}

#[cfg(feature = "cfs")]
static SCHEDULER: &str = "cfs";
#[cfg(feature = "cfs")]
fn scheduler() -> impl Scheduler {
    let (timeslice, remaining, cpu_slices) = arguments();

    println!("Timeslice {timeslice}\nRemaining {remaining}\nCPU slices: {cpu_slices}");
    cfs(NonZeroUsize::new(cpu_slices).unwrap(), remaining)
}

#[cfg(not(any(feature = "round-robin", feature = "priority-queue", feature = "cfs")))]
static SCHEDULER: &str = "no-scheduler";
#[cfg(not(any(feature = "round-robin", feature = "priority-queue", feature = "cfs")))]
fn scheduler() -> impl Scheduler {
    let (timeslice, remaining, cpu_slices) = arguments();

    println!("Timeslice {timeslice}\nRemaining {remaining}\nCPU slices: {cpu_slices}");
    round_robin(NonZeroUsize::new(timeslice).unwrap(), remaining)
}
