use std::{num::NonZeroUsize, process::exit};
use crate::{Scheduler, Process, Pid, ProcessState, StopReason, SchedulingDecision, Syscall, SyscallResult};
use super::pcb::{Pcb, WakeupCondition};

/// Data structure that implements a round robin scheduler.
pub struct RoundRobinScheduler {
    /// The process running on the processor.
    running_process: Option<Pcb>,
    /// Intermediate state a process is in during syscalls.
    stopped_process: Option<Pcb>,
    /// The remaining execution time for the scheduled process.
    remaining_time: usize,
    /// The list of all processes ready to be scheduled.
    ready_processes: Vec<Pcb>,
    /// The list of all processes waiting for an event or sleeping.
    waiting_processes: Vec<Pcb>,
    /// The amount of time a ready process gets on the processor.
    timeslice: NonZeroUsize,
    /// The minimum required time on the processor the stopped process must have remaining
    /// for it to be scheduled imediately after the syscall that stopped it.
    minimum_remaining_timeslice: usize,
    /// The highest pid given to a process.
    highest_pid: usize,
    /// The amount of time the processor needs to sleep for a process to wake up if there are no ready processes to schedule.
    /// Is `0` if there are ready processes.
    sleep_time: usize
}

impl RoundRobinScheduler {

    /// Creates a new [`RoundRobinScheduler`].
    pub fn new(timeslice: NonZeroUsize, minimum_remaining_timeslice: usize) -> Self {
        Self { running_process: None,
            stopped_process: None,
            remaining_time: 0,
            ready_processes: Vec::<Pcb>::new(),
            waiting_processes: Vec::<Pcb>::new(),
            timeslice,
            minimum_remaining_timeslice,
            highest_pid: 0,
            sleep_time: 0
        }
    }

    /// Increments the timings for all processes.
    fn increment_timings(&mut self, _reason: &StopReason) {
        let time = match _reason {
            StopReason::Expired => self.remaining_time,
            StopReason::Syscall { syscall: _, remaining } => self.remaining_time - *remaining
        };

        if let Some(stopped_process) = &mut self.stopped_process {
            match _reason {
                StopReason::Syscall { syscall: _, remaining: _ } => stopped_process.increment_timings(time, 1, time - 1),
                StopReason::Expired => stopped_process.increment_timings(time, 0, time)
            }
        }

        for process in self.ready_processes.iter_mut() {
            process.increment_timings(time, 0, 0);
        }

        for process in self.waiting_processes.iter_mut() {
            process.increment_timings(time, 0, 0);
            if let WakeupCondition::Sleep(sleep_time) = process.wakeup() {
                match sleep_time.checked_sub(time).filter(|remaining_time| *remaining_time != 0) {
                    Some(remaining_time) => 
                        process.set_wakeup(WakeupCondition::Sleep(remaining_time)),
                    None => { 
                        process.set_wakeup(WakeupCondition::None);
                        process.set_state(ProcessState::Ready);
                    }
                }
            }
        }
    }

    /// Moves processes that have waked up into the list of ready processes.
    fn wakeup_processes(&mut self) {
        let mut still_waiting_processes = Vec::<Pcb>::new();
        for process in self.waiting_processes.iter().cloned() {
            if matches!(process.state(), ProcessState::Ready) {
                self.ready_processes.push(process);
            } else {
                still_waiting_processes.push(process);
            }
        }
        self.waiting_processes.clone_from(&still_waiting_processes);
    }

    /// Sleeps for the amount of time needed for a process to become ready for scheduling.
    fn sleep(&mut self) {
        for process in self.waiting_processes.iter_mut() {
            process.increment_timings(self.sleep_time, 0, 0);
            if let WakeupCondition::Sleep(wakeup_time) = process.wakeup() {
                match wakeup_time.checked_sub(self.sleep_time).filter(|remaining_time| *remaining_time != 0) {
                    Some(remaining_time) => 
                        process.set_wakeup(WakeupCondition::Sleep(remaining_time)),
                    None => { 
                        process.set_wakeup(WakeupCondition::None);
                        process.set_state(ProcessState::Ready);
                    }
                }
            }
        }
        self.sleep_time = 0;
        self.wakeup_processes();
    }

    /// Forks a new process with the given priority.
    fn new_process(&mut self, priority: i8) {
        self.highest_pid += 1;
        self.ready_processes.push(Pcb::new(Pid::new(self.highest_pid), priority, 0));
    }
    
    /// Sets a process into the ready state.
    fn set_ready(&mut self, mut process: Pcb) {
        process.set_state(ProcessState::Ready);
        process.set_wakeup(WakeupCondition::None);
        self.ready_processes.push(process);
        self.remaining_time = 0;
    }

    /// Sets a process to into the running state.
    fn set_running(&mut self, mut process: Pcb) {
        process.set_state(ProcessState::Running);
        self.running_process = Some(process);
        self.remaining_time = self.timeslice.get();
    }

    /// Returns `true` if there are no more processes, `false` otherwise.
    fn is_done(&self) -> bool {
        self.running_process.is_none() && self.ready_processes.is_empty() && self.waiting_processes.is_empty()
    }

    /// Returns `true` if the process with pid 1 exists, `false` otherwise.
    fn pid_1_exists(&self) -> bool {
        if let Some(running_process) = &self.running_process {
            if running_process.pid().cmp(&Pid::new(1)).is_eq() {
                return true;
            }
        }
        if self.ready_processes.iter().any(|element| element.pid().cmp(&Pid::new(1)).is_eq()) {
            return true;
        }
        if self.waiting_processes.iter().any(|element| element.pid().cmp(&Pid::new(1)).is_eq()) {
            return true;
        }
        false
    }

    /// Returns the process scheduled to be run.
    fn scheduled_process(&mut self) -> Option<Pcb> {
        if !self.ready_processes.is_empty() {
            Some(self.ready_processes.remove(0))
        } else {
            None
        }
    }

    /// Returns the minimal amount of time the processor needs to sleep for a process to become ready for scheduling.
    fn find_sleep_time(&self) -> Option<usize> {
        let mut minimum_sleep_time: Option<usize> = None;
        for sleep_time in self.waiting_processes.iter().filter_map(|element|
            match element.wakeup() {WakeupCondition::Sleep(sleep_time) => Some(sleep_time), _ => None}) {
            match minimum_sleep_time {
                Some(minimum_sleep_time_value) =>
                    if sleep_time < minimum_sleep_time_value {
                        minimum_sleep_time = Some(sleep_time)
                    },
                None => minimum_sleep_time = Some(sleep_time)
            }
        }
        minimum_sleep_time
    }

    /// Return an vector of refrences to all processes.
    fn get_all_processes(&self) -> Vec<&Pcb> {
        let mut processes = Vec::<&Pcb>::new();
        processes.extend(self.ready_processes.iter());
        processes.extend(self.waiting_processes.iter());
        if let Some(running_process) = &self.running_process {
            processes.push(running_process);
        }
        processes
    }

    /// Handles syscalls recievied from the running process.
    fn syscall_handler(&mut self, syscall: Syscall, remaining_time: usize) -> SyscallResult {
        match syscall {
            Syscall::Fork(priority) => {
                self.new_process(priority);

                self.wakeup_processes();
                match self.stopped_process.take() {
                    Some(mut stopped_process) => {
                        if remaining_time >= self.minimum_remaining_timeslice {
                            stopped_process.set_state(ProcessState::Running);
                            self.running_process = Some(stopped_process);
                            self.remaining_time = remaining_time;
                        } else {
                            self.set_ready(stopped_process)
                        }
                    },
                    None => {
                        self.remaining_time = 0;
                    }
                }

                return SyscallResult::Pid(Pid::new(self.highest_pid));
            }
            Syscall::Signal(event) => {
                for process in self.waiting_processes.iter_mut()
                    .filter(|element| matches!(element.wakeup(), WakeupCondition::Signal(x) if x == event)) {
                    process.set_state(ProcessState::Ready);
                    process.set_wakeup(WakeupCondition::None);
                }

                self.wakeup_processes();
                match self.stopped_process.take() {
                    Some(mut stopped_process) => {
                        if remaining_time >= self.minimum_remaining_timeslice {
                            stopped_process.set_state(ProcessState::Running);
                            self.running_process = Some(stopped_process);
                            self.remaining_time = remaining_time;
                        } else {
                            self.set_ready(stopped_process)
                        }
                    },
                    None => {
                        self.remaining_time = 0;
                    }
                }
            },
            Syscall::Sleep(sleep_time) => {
                match self.stopped_process.take() {
                    Some(mut stopped_process) => {
                        stopped_process.set_state(ProcessState::Waiting { event: None });
                        stopped_process.set_wakeup(WakeupCondition::Sleep(sleep_time));
                        self.waiting_processes.push(stopped_process);
                    },
                    None => return SyscallResult::NoRunningProcess
                }
            },
            Syscall::Wait(event) => {
                match self.stopped_process.take() {
                    Some(mut stopped_process) => {
                        stopped_process.set_state(ProcessState::Waiting { event: Some(event) });
                        stopped_process.set_wakeup(WakeupCondition::Signal(event));
                        self.waiting_processes.push(stopped_process);
                    },
                    None => return SyscallResult::NoRunningProcess
                }
            },
            Syscall::Exit => self.wakeup_processes(),
        };
        
        SyscallResult::Success
    }

}


impl Scheduler for RoundRobinScheduler {
    fn next(&mut self) -> SchedulingDecision {
        if self.sleep_time != 0 {
            self.sleep();
        }

        if self.is_done() {
            return SchedulingDecision::Done;
        }

        if !self.pid_1_exists() {
            return SchedulingDecision::Panic;
        }

        if let Some(scheduled_process) = &mut self.running_process {
            return SchedulingDecision::Run { pid: scheduled_process.pid(), timeslice:
                match NonZeroUsize::new(self.remaining_time) {Some(time) => time, None => exit(-1)}};
        }

        if let Some(scheduled_process) = self.scheduled_process() {
            self.set_running(scheduled_process);
            return SchedulingDecision::Run { pid: match &self.running_process {Some(process) => process.pid(), None => exit(-1)},
            timeslice: self.timeslice };
        }

        match self.find_sleep_time() {
            Some(sleep_time) => {
                self.sleep_time = sleep_time;
                SchedulingDecision::Sleep(match NonZeroUsize::new(sleep_time)
                    {Some(sleep_time) => sleep_time, None => exit(-1)})
            },
            None => SchedulingDecision::Deadlock
        }
    }

    fn stop(&mut self, _reason: StopReason) -> SyscallResult {
        match self.running_process.take() {
            Some(running_process) => self.stopped_process = Some(running_process),
            None => self.stopped_process = None
        }

        self.increment_timings(&_reason);

        match _reason {
            StopReason::Expired => {
                self.wakeup_processes();
                match self.stopped_process.take() {
                    Some(stopped_process) => {
                        self.set_ready(stopped_process);
                        SyscallResult::Success
                    },
                    None => {
                        SyscallResult::NoRunningProcess
                    }
                }
            },
            StopReason::Syscall{ syscall, remaining } => {
                self.syscall_handler(syscall, remaining)
            }
        }
    }

    fn list(&mut self) -> Vec<&dyn Process> {
        let mut processes = self.get_all_processes();

        processes.sort_by_key(|element|  element.pid());

        processes.into_iter().map(|element| element as &dyn Process).collect()
    }
}
