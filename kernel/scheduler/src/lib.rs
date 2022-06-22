#![no_std]
#![feature(let_else)]

extern crate alloc;
#[macro_use] extern crate log;
extern crate irq_safety;
extern crate apic;
extern crate task;
extern crate runqueue;
#[macro_use] extern crate cfg_if;
cfg_if! {
    if #[cfg(priority_scheduler)] {
        extern crate scheduler_priority as scheduler;
    } else if #[cfg(realtime_scheduler)] {
        extern crate scheduler_realtime as scheduler;
    } else {
        extern crate scheduler_round_robin as scheduler;
    }
}

use irq_safety::hold_interrupts;
use apic::current_apic_id;
use task::{try_current_task, TaskRef};

/// Yields the current CPU by selecting a new `Task` to run 
/// and then performs a task switch to that new `Task`.
///
/// Interrupts will be disabled while this function runs.
pub fn yield_now() {
    let _held_interrupts = hold_interrupts(); // auto-reenables interrupts on early return
    let apic_id = current_apic_id();

    let Ok(curr_task) = try_current_task() else {
        error!("BUG: yield_now(): could not get current task.");
        return; // keep running the same current task
    };

    let Some(next_task) = scheduler::select_next_task(apic_id) else {
        return; // keep running the same current task
    };

    // No need to task switch if the chosen task is the same as the current task.
    if &next_task == curr_task {
        return;
    }

    // trace!("BEFORE TASK_SWITCH CALL (AP {}), current: {:?}, next: {:?}, interrupts are {}", apic_id, curr_task, next_task, irq_safety::interrupts_enabled());
    curr_task.task_switch(&next_task, apic_id); 
    // trace!("AFTER TASK_SWITCH CALL (AP {}) new current: {:?}, interrupts are {}", apic_id, current_task(), irq_safety::interrupts_enabled());
}

/// Changes the priority of the given task with the given priority level.
/// Priority values must be between 40 (maximum priority) and 0 (minimum prriority).
/// This function returns an error when a scheduler without priority is loaded. 
pub fn set_priority(_task: &TaskRef, _priority: u8) -> Result<(), &'static str> {
    #[cfg(priority_scheduler)] {
        scheduler_priority::set_priority(_task, _priority)
    }
    #[cfg(not(priority_scheduler))] {
        Err("no scheduler that uses task priority is currently loaded")
    }
}

/// Returns the priority of a given task.
/// This function returns None when a scheduler without priority is loaded.
pub fn get_priority(_task: &TaskRef) -> Option<u8> {
    #[cfg(priority_scheduler)] {
        scheduler_priority::get_priority(_task)
    }
    #[cfg(not(priority_scheduler))] {
        None
    }
}

pub fn set_periodicity(_task: &TaskRef, _period: usize) -> Result<(), &'static str> {
    #[cfg(realtime_scheduler)] {
        scheduler_realtime::set_periodicity(_task, _period)
    }
    #[cfg(not(realtime_scheduler))] {
        Err("no scheduler that supports periodic tasks is currently loaded")
    }
}


