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

//! This module contains the implementation of a single threaded `Future` scheduler.

use crate::core::task::{Id as TaskId, Task};
use once_cell::sync::Lazy;
use std::{collections, fmt, iter, mem, sync, task};

/// Provides the interface to access a `Scheduler` singleton instance. Since the runtime is
/// designed solely for single-threaded environments, all access to the runtime needs to occur
/// via this singleton instance.
struct Singleton;

impl Singleton {
    /// Returns the [`MutexGuard`](https://doc.rust-lang.org/std/sync/struct.MutexGuard.html) of the
    /// `Scheduler` singleton instance.
    #[inline(always)]
    fn instance() -> sync::MutexGuard<'static, Scheduler> {
        static INSTANCE: Lazy<sync::Mutex<Scheduler>> =
            Lazy::new(|| sync::Mutex::new(Scheduler::default()));
        INSTANCE
            .lock()
            .expect("`MutexGuard` of the `Scheduler` singleton should be locked properly")
    }
}

/// Represents the current status of a `Scheduler` instance.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
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

/// The Little Tokio scheduler which is responsible for managing polling tasks.
#[derive(Default)]
pub(crate) struct Scheduler {
    /// Holds the next `Id` value which will be assigned to the next `Task`.
    next_id: TaskId,
    /// Holds the `Task`s to be polled on the Little Tokio runtime.
    pending_tasks: collections::HashMap<TaskId, Task>,
    /// Holds the identifiers of `Task`s ready to be polled.
    scheduled_ids: Vec<TaskId>,
}

impl Scheduler {
    /// Returns the current `Status` of the Little Tokio runtime.
    pub(crate) fn status() -> Status {
        Singleton::instance().get_status()
    }

    /// Returns the scheduled tasks ids to perform further execution.
    pub(crate) fn scheduled_ids() -> impl iter::IntoIterator<Item = TaskId> {
        Singleton::instance().get_scheduled_ids()
    }

    /// Schedules the `task` associated with the given `id` to the scheduler.
    pub(crate) fn schedule(task: Task) {
        Singleton::instance().do_schedule(task);
    }

    /// Notifies the runtime that the `Task` associated with the given `id` is ready to poll.
    pub(crate) fn notify(id: TaskId) {
        Singleton::instance().do_notify(id);
    }

    /// Polls the `Task` associated with a given `id`.
    pub(crate) fn poll(id: TaskId) {
        let task = Singleton::instance().get_task(&id);
        let Some(mut task) = task else {
            return;
        };
        match task
            .as_mut()
            .poll(&mut task::Context::from_waker(&id.into()))
        {
            task::Poll::Pending => {
                Singleton::instance().do_pend(id, task);
            }
            task::Poll::Ready(()) => {}
        }
    }
}

impl Scheduler {
    /// Returns the current `Status` of the Little Tokio runtime.
    fn get_status(&self) -> Status {
        if self.pending_tasks.is_empty() {
            Status::Done
        } else if self.scheduled_ids.is_empty() {
            Status::WaitingForEvents
        } else {
            Status::RunningTasks
        }
    }

    /// Returns the scheduled tasks ids to perform further execution.
    fn get_scheduled_ids(&mut self) -> impl iter::IntoIterator<Item = TaskId> {
        mem::take(&mut self.scheduled_ids)
    }

    /// Returns the next scheduled `Task` to perform further execution.
    fn get_task(&mut self, id: &TaskId) -> Option<Task> {
        self.pending_tasks.remove(id)
    }

    /// Schedules the `task` to the scheduler.
    fn do_schedule(&mut self, task: Task) {
        let id = self.next_id.increment();
        self.pending_tasks.insert(id, task);
        self.scheduled_ids.push(id);
    }

    /// Pends the `task` associated with the given `id` to the scheduler.
    fn do_pend(&mut self, id: TaskId, task: Task) {
        self.pending_tasks.insert(id, task);
    }

    /// Notifies the runtime that the `Task` associated with the given `id` is ready to poll.
    fn do_notify(&mut self, id: TaskId) {
        self.scheduled_ids.push(id);
    }
}
