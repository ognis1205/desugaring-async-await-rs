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
mod net;
mod sys;

use crate::core::runtime::Runtime;
use crate::core::runtime::Status;
use crate::core::runtime::RUNTIME;
use std::{future, mem};

/// Runs a `Future` to completion on the Little Tokio runtime. This is the runtime’s entry point.
pub fn block_on(main_task: impl future::Future<Output = ()> + 'static) {
    // Instanciates one runtime per thread.
    RUNTIME.with_borrow_mut(|runtime| {
        if runtime.is_some() {
            panic!("can not spawn more than 1 runtime on the same thread");
        }
        *runtime = Some(Runtime::default());
    });
    // Spawns the main task.
    spawn(main_task);
    // Performs the task execution if there are tasks that can be processed. Otherwise, turns the event loop.
    loop {
        let scheduled_ids = RUNTIME
            .with_borrow_mut(|runtime| mem::take(&mut runtime.as_mut().unwrap().scheduled_ids));
        for id in scheduled_ids {
            RUNTIME.with_borrow_mut(|runtime| runtime.as_mut().unwrap().poll(id));
        }
        match RUNTIME.with_borrow(|runtime| runtime.as_ref().unwrap().status()) {
            Status::RunningTasks => continue,
            Status::WaitingForEvents => {
                RUNTIME
                    .with_borrow_mut(|runtime| runtime.as_mut().unwrap().try_turn())
                    .expect("");
            }
            Status::Done => return,
        }
    }
    // Removes the injected data from the runtime thread.
    RUNTIME.take();
}

/// Spawns a future onto the Little Tokio runtime.
pub fn spawn(task: impl future::Future<Output = ()> + 'static) {
    let task = Box::pin(task);
    RUNTIME.with_borrow_mut(|runtime| {
        let runtime = runtime.as_mut().unwrap();
        let id = runtime.next_id.increment();
        runtime.tasks.insert(id, task);
        runtime.schedule(id);
    });
}
