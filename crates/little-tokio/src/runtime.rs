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

//! This module contains the implementation of a single threaded `Future` runtime.

use crate::task::{Id as TaskId, Task};
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;

thread_local! {
    /// Provides the interface to access a `Runtime` thread-local instance. Since the runtime is
    /// designed solely for single-threaded environments, All access to the runtime needs to occur
    /// via this thread-local instance.
    pub(crate) static RUNTIME: RefCell<Option<Runtime>> = RefCell::new(None);
}

/// The Little Tokio runtime which is responsible for I/O multiplexing.
#[derive(Default)]
pub struct Runtime {
    /// Holds the `Task`s to be polled on the Little Tokio runtime.
    pub(crate) tasks: HashMap<TaskId, Task>,
    /// Holds the identifiers of `Task`s ready to be polled.
    pub(crate) scheduled_ids: Vec<TaskId>,
}

impl Runtime {
    /// Runs a `Future` to completion on the Little Tokio runtime. This is the runtime’s entry point.
    pub fn block_on(main_task: impl Future<Output = ()> + 'static) {
        todo!()
    }

    /// Spawns a future onto the Little Tokio runtime.
    pub fn spawn(task: impl Future<Output = ()> + 'static) {
        todo!()
    }
}
