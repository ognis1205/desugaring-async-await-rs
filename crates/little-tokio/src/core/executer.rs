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

//! This module contains the implementation of a single threaded `Future` executer.

use crate::core::task::{Id as TaskId, Task};
use std::{collections, fmt, task};

/// Represents the current status of a `Executer` instance.
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

/// The Little Tokio executer which is responsible for I/O multiplexing.
#[derive(Default)]
pub(crate) struct Executer {
    /// Holds the next `Id` value which will be assigned to the next `Task`.
    pub(crate) next_id: TaskId,
    /// Holds the `Task`s to be polled on the Little Tokio runtime.
    pub(crate) pending_tasks: collections::HashMap<TaskId, Task>,
    /// Holds the identifiers of `Task`s ready to be polled.
    pub(crate) scheduled_ids: Vec<TaskId>,
}

impl Executer {
    /// Polls the `Task` associated with a given `id`.
    #[inline]
    pub(crate) fn poll(&mut self, id: TaskId) {
        let task = self.pending_tasks.remove(&id);
        let Some(mut task) = task else {
            return;
        };
        match task
            .as_mut()
            .poll(&mut task::Context::from_waker(&id.into()))
        {
            task::Poll::Pending => {
                self.pending_tasks.insert(id, task);
            }
            task::Poll::Ready(()) => {}
        }
    }

    /// Returns the current `Status` of a `Executer`.
    #[inline]
    pub(crate) fn status(&self) -> Status {
        if self.pending_tasks.is_empty() {
            return Status::Done;
        } else if self.scheduled_ids.is_empty() {
            return Status::WaitingForEvents;
        } else {
            return Status::RunningTasks;
        }
    }
}
