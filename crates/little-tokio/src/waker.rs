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

//! This module contains the implementation of a waker.

use crate::scheduler::Scheduler;
use crate::task::Id as TaskId;
use std::task::{RawWaker, RawWakerVTable, Waker};

/// The current design of the [`Waker`](https://doc.rust-lang.org/std/task/struct.Waker.html)
/// is focused on performance and embedded-like scenarios. Hence, This wake-related vtable
/// functions will be associated with a data which will be required when `Scheduler` schedules
/// a `Task`.
static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

/// This function will be called when the 'Waker' gets cloned and creates a new `RawWaker` from
/// the provided data pointer, i.e., an `Id`, and vtable.
unsafe fn clone(id: *const ()) -> RawWaker {
    RawWaker::new(id, &VTABLE)
}

/// This function will be called when `wake` is called on the `Waker` and schedules the `Task`
/// associated with a give `id`.
unsafe fn wake(id: *const ()) {
    wake_by_ref(id);
}

/// This function will be called when `wake_by_ref` is called on the `Waker` and schedules the `Task`
/// associated with a give `id`.
unsafe fn wake_by_ref(id: *const ()) {
    Scheduler::schedule(TaskId::from_ptr(id))
}

/// This function gets called when a `Waker` gets dropped.
unsafe fn drop(_id: *const ()) {
    // Do nothing.
}

impl From<TaskId> for Waker {
    fn from(id: TaskId) -> Self {
        unsafe { Self::from_raw(RawWaker::new(id.to_ptr(), &VTABLE)) }
    }
}
