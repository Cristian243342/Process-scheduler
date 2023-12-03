use crate::{ProcessState, Pid, Process};

/// Enumerates the possible wakeup condition for [Pcb].
#[derive(Clone, Copy)]
pub enum WakeupCondition {
    /// Contains the amount of time units the process needs to sleep.
    Sleep(usize),
    /// Contains the event number of the event the process is waiting for.
    Signal(usize),
    /// The process isn't waiting.
    None
}

/// Data structure that implements a process.
#[derive(Clone)]
pub struct Pcb {
    /// The process's pid.
    pid: Pid,
    process_state: ProcessState,
    /// A set of times associated with the process, updated dynamically during process execution.
    /// * `timings.0` - the total time from the process's inital fork to exit.
    /// * `timings.1` - the time spent on syscalls.
    /// * `timings.2` - the execution time (time spent on the processor).
    timings: (usize, usize, usize),
    /// The condition for a waiting process to wake up.
    wakeup: WakeupCondition,
    /// The initial priority given to the process when it was forked.
    fork_priority: i8,
    /// The priority of the process.
    priority: i8,
    /// Extra information about the process.
    extra: String
}

impl Pcb {
    
    /// Creates a new [`Pcb`].
    /// 
    /// The defaults are as follows:
    /// * process_state: [`ProcessState::Ready`]
    /// * timings: `(0, 0, 0)`
    /// * extra: `String::from("")`
    pub fn new(pid: Pid, priority: i8) -> Self {
        Self { pid,
               process_state: ProcessState::Ready,
               timings: (0, 0, 0),
               wakeup: WakeupCondition::None,
               fork_priority: priority,
               priority: priority,
               extra: String::from("")
        }
    }

    /// Sets the process state of a [`Pcb`].
    pub fn set_state(&mut self, state: ProcessState) {
        self.process_state = state;
    }

    /// Returns the wakeup condition of a [`Pcb`].
    pub fn wakeup(&self) -> WakeupCondition {
        self.wakeup.clone()
    }

    /// Sets the wakeup condition of a [`Pcb`].
    pub fn set_wakeup(&mut self, wakeup: WakeupCondition) {
        self.wakeup = wakeup;
    }

    /// Increments the timings of a [`Pcb`] by the specified values.
    /// ### Parameters
    /// * total_time: Increments the [`Pcb`]'s total time by this value;
    /// * syscall_time: Increments the [`Pcb`]'s syscall time by this value;
    /// * execution_time: Increments the [`Pcb`]'s execution time by this value.
    pub fn increment_timings(&mut self, total_time: usize, syscall_time: usize, execution_time: usize) {
        self.timings.0 += total_time;
        self.timings.1 += syscall_time;
        self.timings.2 += execution_time;
    }

    /// Increments the priority of a [`Pcb`], but not over its fork priority.
    pub fn increment_priority(&mut self) {
        if self.priority != self.fork_priority {
            self.priority += 1;
        }
    }

    /// Decrements the priority of a [`Pcb`], but not under `0`.
    pub fn decrement_priority(&mut self) {
        if self.priority != 0 {
            self.priority -= 1;
        }
    }

}

impl Process for Pcb {
    fn pid(&self) -> Pid {
        self.pid.clone()
    }

    fn state(&self) -> ProcessState {
        self.process_state.clone()
    }

    fn timings(&self) -> (usize, usize, usize) {
        self.timings.clone()
    }

    fn priority(&self) -> i8 {
        self.priority.clone()
    }

    fn extra(&self) -> String {
        self.extra.clone()
    }
}
