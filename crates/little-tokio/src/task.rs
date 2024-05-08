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

//! This module contains the implementation of a `Task` which represents the unit of
//! comupation (state machine) of the `Runtime`, i.e., a `Future` implementation.

use crate::vtable::VTABLE;
use std::future::Future;
use std::pin::Pin;
use std::task::{RawWaker, Waker};

/// Represents a `Task` of `Runtime` is defined as a heap-allocated and `Pin`ned instance of the `Future`.
pub(crate) type Task = Pin<Box<dyn Future<Output = ()>>>;

/// Specifies the identifier of a `Task`, which is defined as an `usize` number. In theory, tasks can
/// have arbitrary data types which will be used for the future usage of a `Future` runtime. However,
/// the `Runtime` of this crate assumes that only `Id` values are allowed for the data since this crate
/// is for self-studying purpose.
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) struct Id(usize);

impl Id {
    /// The current design of the [`Waker`](https://doc.rust-lang.org/std/task/struct.Waker.html)
    /// is focused on performance and embedded-like scenarios. Hence, the `Id` value, which is
    /// a data associated with the wake-related vtable functions, will be accessed via its raw pointer.
    pub(crate) fn to_ptr(self) -> *const () {
        self.0 as _
    }

    /// The current design of the [`Waker`](https://doc.rust-lang.org/std/task/struct.Waker.html)
    /// is focused on performance and embedded-like scenarios. Hence, the `Id` value, which is
    /// a data associated with the wake-related vtable functions, will be accessed via its raw pointer.
    pub(crate) fn from_ptr(value: *const ()) -> Self {
        Self(value as _)
    }
}

impl From<Id> for Waker {
    fn from(id: Id) -> Self {
        // SAFETY:
        // Given that the implementation of this runtime aims to provide a single-threaded version of
        // an I/O multiplexer, this restriction is lifted
        unsafe { Self::from_raw(RawWaker::new(id.to_ptr(), &VTABLE)) }
    }
}
