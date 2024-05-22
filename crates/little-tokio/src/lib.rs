// Copyright 2024 Shingo OKAWA and a number of other contributors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This crate contains a minimal implementation of a Rust `Future` runtime.
//! The implementation is for self-study purpose only, so there might be some
//! issues. Please use this crate at your own risk.

mod core;
pub mod net;
mod sys;

use crate::core::runtime::RUNTIME;
use crate::core::task::{Id as TaskId, Task};
use std::{cell, collections, fmt, future, mem, task};

thread_local! {
    /// Provides the interface to access a `Schedule` thread-local instance. Since the runtime is
    /// designed solely for single-threaded environments, all access to the schedule needs to occur
    /// via this thread-local instance.
    pub(crate) static SCHEDULE: cell::RefCell<Option<Schedule>> = cell::RefCell::new(None);
}

/// Represents the current status of a `Schedule` instance.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    /// Specifies when the executer is polling scheduled tasks.
    RunningTasks,
    /// Specifies when the executer is turning the event loop and waiting for the next events.
    WaitingForEvents,
    /// Specifies when all operations of the runtime have completed.
    Done,
}

impl fmt::Debug for Status {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RunningTasks => write!(fmt, "Status::RunningTasks")?,
            Self::WaitingForEvents => write!(fmt, "Status::WaitingForEvents")?,
            Self::Done => write!(fmt, "Status::Done")?,
        }
        Ok(())
    }
}

/// The Little Tokio schedule which is responsible for managing polling tasks.
#[derive(Default)]
pub(crate) struct Schedule {
    /// Holds the next `Id` value which will be assigned to the next `Task`.
    pub(crate) next_id: TaskId,
    /// Holds the `Task`s to be polled on the Little Tokio runtime.
    pub(crate) pending_tasks: collections::HashMap<TaskId, Task>,
    /// Holds the identifiers of `Task`s ready to be polled.
    pub(crate) scheduled_ids: Vec<TaskId>,
}

/// Runs a `Future` to completion on the Little Tokio runtime. This is the runtime’s entry point.
pub fn block_on(main_task: impl future::Future<Output = ()> + 'static) {
    // Instanciates one schedule per thread.
    SCHEDULE.with_borrow_mut(|schedule| {
        if schedule.is_some() {
            panic!("can not spawn more than one schedule on the same thread");
        }
        *schedule = Some(Schedule::default());
    });
    // Spawns the main task.
    spawn(main_task);
    // Performs the task execution if there are tasks that can be processed. Otherwise, turns the event loop.
    loop {
        let scheduled_ids = SCHEDULE
            .with_borrow_mut(|schedule| mem::take(&mut schedule.as_mut().unwrap().scheduled_ids));
        for id in scheduled_ids {
            poll(id);
        }
        match status() {
            Status::RunningTasks => continue,
            Status::WaitingForEvents => {
                RUNTIME
                    .with_borrow_mut(|runtime| {
                        runtime
                            .as_mut()
                            .expect("should acquire runtime properly")
                            .try_turn()
                    })
                    .expect("should turn the event loop properly");
            }
            Status::Done => break,
        }
    }
    // Removes the injected data from the runtime thread.
    SCHEDULE.take();
}

/// Spawns a future onto the Little Tokio runtime.
pub fn spawn(task: impl future::Future<Output = ()> + 'static) {
    let task = Box::pin(task);
    SCHEDULE.with_borrow_mut(|schedule| {
        let Some(schedule) = schedule else {
            panic!("runtime should be initialized before running tasks");
        };
        let id = schedule.next_id.increment();
        schedule.pending_tasks.insert(id, task);
        schedule.scheduled_ids.push(id);
    });
}

/// Polls the `Task` associated with a given `id`.
fn poll(id: TaskId) {
    let task =
        SCHEDULE.with_borrow_mut(|schedule| schedule.as_mut().unwrap().pending_tasks.remove(&id));
    let Some(mut task) = task else {
        return;
    };
    match task
        .as_mut()
        .poll(&mut task::Context::from_waker(&id.into()))
    {
        task::Poll::Pending => {
            SCHEDULE.with_borrow_mut(|schedule| {
                schedule.as_mut().unwrap().pending_tasks.insert(id, task);
            });
        }
        task::Poll::Ready(()) => {}
    }
}

/// Returns the current `Status` of the Little Tokio runtime.
fn status() -> Status {
    SCHEDULE.with_borrow(|schedule| {
        let schedule = schedule.as_ref().unwrap();
        if schedule.pending_tasks.is_empty() {
            return Status::Done;
        } else if schedule.scheduled_ids.is_empty() {
            return Status::WaitingForEvents;
        } else {
            return Status::RunningTasks;
        }
    })
}
