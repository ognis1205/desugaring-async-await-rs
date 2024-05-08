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

//! This module contains the implementation of a vtable for dispatching methods on `Waker`.

use crate::core::CORE;
use crate::task::Id as TaskId;
use std::task::{RawWaker, RawWakerVTable};

/// The current design of the [`Waker`](https://doc.rust-lang.org/std/task/struct.Waker.html)
/// is focused on performance and embedded-like scenarios. Hence, This wake-related vtable
/// functions will be associated with a data which will be required when `Scheduler` schedules
/// a `Task`.
pub(crate) static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

/// This function will be called when the 'Waker' gets cloned and creates a new `RawWaker` from
/// the provided data pointer, i.e., an `Id`, and vtable.
///
/// SAFETY:
/// Given that the implementation of this runtime aims to provide a single-threaded version of
/// an I/O multiplexer, this restriction is lifted
unsafe fn clone(id: *const ()) -> RawWaker {
    RawWaker::new(id, &VTABLE)
}

/// This function will be called when `wake` is called on the `Waker` and schedules the `Task`
/// associated with a give `id`.
///
/// SAFETY:
/// Given that the implementation of this runtime aims to provide a single-threaded version of
/// an I/O multiplexer, this restriction is lifted
unsafe fn wake(id: *const ()) {
    wake_by_ref(id);
}

/// This function will be called when `wake_by_ref` is called on the `Waker` and schedules the `Task`
/// associated with a give `id`.
///
/// SAFETY:
/// Given that the implementation of this runtime aims to provide a single-threaded version of
/// an I/O multiplexer, this restriction is lifted
unsafe fn wake_by_ref(id: *const ()) {
    CORE.with_borrow_mut(|core| {
        core.as_mut().unwrap().schedule(TaskId::from_ptr(id));
    })
}

/// This function gets called when a `Waker` gets dropped.
///
/// SAFETY:
/// Given that the implementation of this runtime aims to provide a single-threaded version of
/// an I/O multiplexer, this restriction is lifted
unsafe fn drop(_id: *const ()) {
    // Do nothing.
}
