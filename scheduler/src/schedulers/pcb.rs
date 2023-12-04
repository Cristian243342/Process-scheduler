use std::{ops::{AddAssign, Add}, process::exit};

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
    vruntime: usize,
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
    pub fn new(pid: Pid, priority: i8, vruntime: usize) -> Self {
        Self { pid,
               process_state: ProcessState::Ready,
               timings: (0, 0, 0),
               vruntime,
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

    pub fn vruntime(&self) -> usize {
        self.vruntime
    }

    /// Returns the wakeup condition of a [`Pcb`].
    pub fn wakeup(&self) -> WakeupCondition {
        self.wakeup.clone()
    }

    /// Sets the wakeup condition of a [`Pcb`].
    pub fn set_wakeup(&mut self, wakeup: WakeupCondition) {
        self.wakeup = wakeup;
    }

    pub fn set_extra(&mut self, extra: String) {
        self.extra = extra;
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

impl Add<usize> for Pcb {
    type Output = usize;
    fn add(self, value: usize) -> <Self as Add<usize>>::Output {
        return self.vruntime + value;
    }
}

impl AddAssign<usize> for Pcb {
    fn add_assign(&mut self, value: usize) {
        self.vruntime += value;
    }
}

impl PartialEq for Pcb {
    fn eq(&self, other: &Self) -> bool {
        self.vruntime.eq(&other.vruntime)
    }
}

impl PartialOrd for Pcb {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.eq(other) {
            return Some(self.pid.cmp(&other.pid));
        }
        Some(self.vruntime.cmp(&other.vruntime))
    }
}

impl Eq for Pcb {
}

impl Ord for Pcb {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.partial_cmp(other) {
            Some(order) => order,
            None => exit(-1)
        }
    }
}
