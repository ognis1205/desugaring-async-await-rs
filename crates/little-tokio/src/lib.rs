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
pub mod utils;

use crate::core::reactor::Reactor;
use crate::core::scheduler::{Scheduler, Status};
use std::{future, marker};

/// Runs a `Future` to completion on the Little Tokio runtime. This is the runtime’s entry point.
pub fn block_on(main: impl future::Future<Output = ()> + marker::Send + 'static) {
    // Spawns the main task.
    spawn(main);
    // Performs the task execution if there are tasks that can be processed. Otherwise, turns the event loop.
    loop {
        for id in Scheduler::scheduled_ids() {
            Scheduler::poll(id);
        }
        match Scheduler::status() {
            Status::RunningTasks => continue,
            Status::WaitingForEvents => Reactor::turn(),
            Status::Done => break,
        }
    }
}

/// Spawns a future onto the Little Tokio runtime.
pub fn spawn(task: impl future::Future<Output = ()> + marker::Send + 'static) {
    Scheduler::schedule(Box::pin(task));
}
