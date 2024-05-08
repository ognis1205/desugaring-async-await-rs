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

//! This module contains the implementation of a `Future` scheduler.

use crate::runtime::RUNTIME;
use crate::task::Id as TaskId;

/// Represents a scheduler. While the crate provides only single-threaded I/O multiplexing runtime,
/// this done not necessarily need to be empty struct. However, we implemented it this way because
/// of a nuance: the static method collection associated with this struct behaves like a scheduler
/// instance method.
pub(crate) struct Scheduler;

/// Represents the current status of a `Scheduler` instance.
pub(crate) enum Status {
    RunningTasks,
    WaitingForIOEvents,
    Done,
}

impl Scheduler {
    /// Schedules the `Task` associated with a given `id` to be ready to poll.
    pub(crate) fn schedule(id: TaskId) {
        RUNTIME.with_borrow_mut(|runtime| {
            runtime.as_mut().unwrap().scheduled_ids.push(id);
        })
    }

    /// Returns the current `Status` of a `Scheduler`.
    pub(crate) fn status() -> Status {
        RUNTIME.with_borrow(|runtime| {
            let runtime = runtime.as_ref().unwrap();
            if runtime.tasks.is_empty() {
                return Status::Done;
            } else if runtime.scheduled_ids.is_empty() {
                return Status::WaitingForIOEvents;
            } else {
                return Status::RunningTasks;
            }
        })
    }
}
