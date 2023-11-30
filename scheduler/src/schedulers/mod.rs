//! Implement the schedulers in this module
//!
//! You might want to create separate files
//! for each scheduler and export it here
//! like
//!
//! ```ignore
//! mod scheduler_name
//! pub use scheduler_name::SchedulerName;
//! ```
//!

mod pcb;

mod empty;
pub use empty::Empty;

mod round_robin;
pub use round_robin::RoundRobinScheduler;

// TODO import your schedulers here
