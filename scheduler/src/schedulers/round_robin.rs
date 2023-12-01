use std::{num::NonZeroUsize, process::exit};
use crate::{Scheduler, Process, Pid, ProcessState, StopReason, SchedulingDecision, Syscall, SyscallResult};
use super::pcb::{PCB, WakeupCondition};

pub struct RoundRobinScheduler {
    running_process: Option<Box<PCB>>,
    remaining_time: usize,
    ready_processes: Vec<Box<PCB>>,
    waiting_processes: Vec<Box<PCB>>,
    timeslice: NonZeroUsize,
    minimum_remaining_timeslice: usize,
    highest_pid: usize
}

impl RoundRobinScheduler {
    pub fn new(timeslice: NonZeroUsize, minimum_remaining_timeslice: usize) -> Self {
        Self { running_process: None,
            remaining_time: timeslice.get(),
            ready_processes: Vec::<Box<PCB>>::new(),
            waiting_processes: Vec::<Box<PCB>>::new(),
            timeslice: timeslice,
            minimum_remaining_timeslice: minimum_remaining_timeslice,
            highest_pid: 0
        }
    }

    fn wakeup_processes(&mut self) {
        for index in 0..self.waiting_processes.len() {
            if matches!(self.waiting_processes[index].state(), ProcessState::Ready) {
                self.ready_processes.push(self.waiting_processes.remove(index));
            }
        }
    }

    fn increment_timings(&mut self, time: usize) {
        if let Some(running_process) = &mut self.running_process {
            running_process.increment_timings(time, 0, time);
        }
        for process in self.ready_processes.iter_mut() {
            process.increment_timings(time, 0, 0);
        }

        for process in self.waiting_processes.iter_mut() {
            process.increment_timings(time, time, 0);
            if let WakeupCondition::Sleep(sleep_time) = process.wakeup() {
                match sleep_time.checked_sub(time) {
                    Some(remaining_time) => process.set_wakeup(WakeupCondition::Sleep(remaining_time)),
                    None => { 
                        process.set_wakeup(WakeupCondition::None);
                        process.set_state(ProcessState::Ready);
                    }
                }
            }
        }
    }

    fn syscall_handler(&mut self, syscall: Syscall, remaining_time: usize) -> SyscallResult {
        if let Syscall::Fork(priority) = syscall {
            self.highest_pid += 1;
            match self.running_process.take() {
                Some(mut running_process) => {
                    self.ready_processes.push(Box::new(PCB::new(Pid::new(self.highest_pid), priority,
                    String::from("Forked from PPID ") + running_process.pid().to_string().as_str())));

                    if remaining_time >= self.minimum_remaining_timeslice {
                        self.running_process = Some(running_process);
                        self.remaining_time = remaining_time;
                    } else {
                        running_process.set_state(ProcessState::Ready);
                        self.ready_processes.push(running_process);
                    }
                },
                None => {
                    let mut init_process = Box::new(PCB::new(Pid::new(self.highest_pid), priority,String::from("Init")));
                    init_process.set_state(ProcessState::Running);
                    self.running_process = Some(init_process);
                }
            }
            
            return SyscallResult::Pid(Pid::new(self.highest_pid));
        }

        match self.running_process.take() {
            None => SyscallResult::NoRunningProcess,
            Some(mut running_process) => {
                match syscall {
                    Syscall::Signal(event) => {
                        for process in self.waiting_processes.iter_mut()
                            .filter(|element| matches!(element.wakeup(), WakeupCondition::Signal(x) if x == event)) {
                            process.set_state(ProcessState::Ready);
                            process.set_wakeup(WakeupCondition::None);
                        }
                        self.wakeup_processes();
                    },
                    Syscall::Sleep(sleep_time) => {
                        running_process.set_state(ProcessState::Waiting { event: Some(sleep_time) });
                        running_process.set_wakeup(WakeupCondition::Sleep(sleep_time));
                        self.waiting_processes.push(running_process);
                    },
                    Syscall::Wait(event) => {
                        running_process.set_state(ProcessState::Waiting { event: Some(event) });
                        running_process.set_wakeup(WakeupCondition::Signal(event));
                        self.waiting_processes.push(running_process);
                    },
                    _ => (),
                    }
                
                
                SyscallResult::Success
            }
        }
    }

}

impl Scheduler for RoundRobinScheduler {
    fn next(&mut self) -> SchedulingDecision {
        //self.increment_timings(1);

        if self.running_process.is_none() && self.ready_processes.is_empty() && self.waiting_processes.is_empty() {
            return SchedulingDecision::Done;
        }

        let mut pid_1_is_running = false;

        if let Some(running_process) = &mut self.running_process {
            if running_process.pid().cmp(&Pid::new(1)).is_eq() {
                pid_1_is_running = true;
            }
        }
        if self.ready_processes.iter().find(|element| element.pid().cmp(&Pid::new(1)).is_eq()).is_some() {
            pid_1_is_running = true;
        }
        if self.waiting_processes.iter().find(|element| element.pid().cmp(&Pid::new(1)).is_eq()).is_some() {
            pid_1_is_running = true;
        }

        if !pid_1_is_running {
            return SchedulingDecision::Panic;
        }

        if let Some(running_process) = &mut self.running_process {
            return SchedulingDecision::Run { pid: running_process.pid(), timeslice:
                match NonZeroUsize::new(self.remaining_time) {Some(time) => time, None => exit(-1)}};
        }

        if let Some(next_process) = self.ready_processes.get_mut(0) {
            next_process.set_state(ProcessState::Running);
            self.running_process = Some(self.ready_processes.remove(0));
            self.remaining_time = self.timeslice.get();
            return SchedulingDecision::Run { pid: match &self.running_process {Some(process) => process.pid(), None => exit(-1)},
            timeslice: match NonZeroUsize::new(self.remaining_time) {Some(time) => time, None => exit(-1)}};
        }

        let mut minimum_sleep_time: Option<usize> = None;
        for sleep_time in self.waiting_processes.iter().filter_map(|element|
            match element.wakeup() {WakeupCondition::Sleep(sleep_time) => Some(sleep_time), _ => None}) {
            match minimum_sleep_time {
                Some(minimum_sleep_time_value) => if sleep_time < minimum_sleep_time_value { minimum_sleep_time = Some(sleep_time) },
                None => minimum_sleep_time = Some(sleep_time)
            }
        }

        match minimum_sleep_time {
            Some(sleep_time) => return SchedulingDecision::Sleep(match NonZeroUsize::new(sleep_time)
                {Some(sleep_time) => sleep_time, None => exit(-1)}),
            None => return SchedulingDecision::Deadlock
        }
    }

    fn stop(&mut self, _reason: StopReason) -> SyscallResult {
        //self.increment_timings(1);

        match _reason {
            StopReason::Expired => self.increment_timings(self.timeslice.get()),
            StopReason::Syscall { syscall: _, remaining } => self.increment_timings(self.timeslice.get() - remaining)
        }

        match _reason {
            StopReason::Expired =>
                match self.running_process.take() {
                    Some(running_process) => {
                        self.ready_processes.push(running_process);
                        
                        SyscallResult::Success
                    },
                    None => {
                        
                        SyscallResult::NoRunningProcess
                    }
                },
            StopReason::Syscall{ syscall, remaining } => {
                self.syscall_handler(syscall, remaining)
            }
        }
    }


    fn list(&mut self) -> Vec<&dyn Process> {
        //self.increment_timings(1);

        let mut processes = Vec::<&Box<PCB>>::new();
        processes.extend(self.ready_processes.iter());
        processes.extend(self.waiting_processes.iter());
        if let Some(running_process) = &self.running_process {
            processes.push(running_process);
        }

        processes.sort_by(|element1, element2|  element1.pid().cmp(&element2.pid()));

        return processes.into_iter().map(|element| element.as_ref() as &dyn Process).collect();
    }
}