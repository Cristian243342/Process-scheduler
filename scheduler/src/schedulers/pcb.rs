use crate::{ProcessState, Pid, Process};

#[derive(Clone, Copy)]
pub enum WakeupCondition {
    Sleep(usize),
    Signal(usize),
    None
}

pub struct PCB {
    pid: Pid,
    process_state: ProcessState,
    timings: (usize, usize, usize),
    wakeup: WakeupCondition,
    priority: i8,
    extra: String
}

impl PCB {
    pub fn new(pid: Pid, priority: i8, extra: String) -> Self {
        Self { pid,
               process_state: ProcessState::Ready,
               timings: (0,0,0),
               wakeup: WakeupCondition::None,
               priority,
               extra
        }
    }

    pub fn wakeup(&self) -> WakeupCondition {
        self.wakeup.clone()
    }

    // pub fn set_pid(&mut self, pid: usize) {
    //     self.pid = Pid::new(pid);
    // }

    pub fn set_state(&mut self, state: ProcessState) {
        self.process_state = state;
    }

    pub fn set_wakeup(&mut self, wakeup: WakeupCondition) {
        self.wakeup = wakeup;
    }

    // pub fn set_priority(&mut self, priority: i8) {
    //     self.priority = priority;
    // }

    // pub fn set_extra(&mut self, extra: String) {
    //     self.extra = extra;
    // }

    pub fn increment_timings(&mut self, total_time: usize, syscall_time: usize, execution_time: usize) {
        self.timings.0 += total_time;
        self.timings.1 += syscall_time;
        self.timings.2 += execution_time;
    }

}

impl Process for PCB {
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
