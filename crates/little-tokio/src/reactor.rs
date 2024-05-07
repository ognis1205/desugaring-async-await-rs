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

//! This module contains the implementation of an I/O reactor.

use crate::runtime::RUNTIME;
use crate::task::Id as TaskId;

/// Represents a reactor. While the crate provides only single-threaded I/O multiplexing runtime,
/// this done not necessarily need to be empty struct. However, we implemented it this way because
/// of a nuance: the static method collection associated with this struct behaves like a reactor
/// instance method.
pub(crate) struct Reactor;

impl Reactor {
    /// Polls the `Task` associated with a given `id`.
    pub(crate) fn run(id: TaskId) {
        todo!()
    }

    /// Performs one iteration of the I/O event loop.
    pub(crate) fn turn() {
        todo!()
    }
}
