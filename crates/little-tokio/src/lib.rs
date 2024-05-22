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

use crate::core::executer::{Executer, Status};
use crate::core::runtime::RUNTIME;
use std::{cell, future, mem};

thread_local! {
    /// Provides the interface to access a `Executer` thread-local instance. Since the runtime is
    /// designed solely for single-threaded environments, all access to the executer needs to occur
    /// via this thread-local instance.
    pub(crate) static EXECUTER: cell::RefCell<Option<Executer>> = cell::RefCell::new(None);
}

/// Runs a `Future` to completion on the Little Tokio runtime. This is the runtime’s entry point.
pub fn block_on(main_task: impl future::Future<Output = ()> + 'static) {
    // Instanciates one executer per thread.
    EXECUTER.with_borrow_mut(|executer| {
        if executer.is_some() {
            panic!("can not spawn more than one executer on the same thread");
        }
        *executer = Some(Executer::default());
    });
    // Spawns the main task.
    spawn(main_task);
    // Performs the task execution if there are tasks that can be processed. Otherwise, turns the event loop.
    loop {
        let scheduled_ids = EXECUTER
            .with_borrow_mut(|executer| mem::take(&mut executer.as_mut().unwrap().scheduled_ids));
        for id in scheduled_ids {
            EXECUTER.with_borrow_mut(|executer| executer.as_mut().unwrap().poll(id));
        }
        let status = EXECUTER.with_borrow(|executer| executer.as_ref().unwrap().status());
        match status {
            Status::RunningTasks => continue,
            Status::WaitingForEvents => {
                RUNTIME
                    .with_borrow_mut(|runtime| {
                        runtime
                            .as_mut()
                            .expect("should acquire runtime properly")
                            .try_turn()
                    })
                    .expect("should turn the event loop properly");
            }
            Status::Done => break,
        }
    }
    // Removes the injected data from the runtime thread.
    EXECUTER.take();
}

/// Spawns a future onto the Little Tokio runtime.
pub fn spawn(task: impl future::Future<Output = ()> + 'static) {
    let task = Box::pin(task);
    EXECUTER.with_borrow_mut(|executer| {
        let Some(executer) = executer else {
            panic!("runtime should be initialized before running tasks");
        };
        let id = executer.next_id.increment();
        executer.pending_tasks.insert(id, task);
        executer.scheduled_ids.push(id);
    });
}
