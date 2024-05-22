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

use crate::core::interest::Interest;
use crate::core::token::Token;
use crate::sys::unix::kqueue::Events;
use crate::sys::unix::kqueue::Selector;
use std::{cell, collections, fmt, io, os, task};

thread_local! {
    /// Provides the interface to access a `Runtime` thread-local instance. Since the runtime is
    /// designed solely for single-threaded environments, all access to the runtime needs to occur
    /// via this thread-local instance.
    pub(crate) static RUNTIME: cell::RefCell<Option<Runtime>> = cell::RefCell::new(None);
}

/// The Little Tokio runtime which is responsible for I/O multiplexing.
#[derive(Default)]
pub(crate) struct Runtime {
    /// Holds the `libc::kqueue` based IO demultiplexer.
    pub(crate) selector: Selector,
    /// Holds the correspondence between blocked file descriptors' tokens and their corresponding wakers, which
    /// the runtime utilizes to wake up tasks.
    pub(crate) blocked_fds: collections::HashMap<Token, task::Waker>,
}

impl Runtime {
    /// Performs one iteration of the I/O event loop.
    pub(crate) fn try_turn(&mut self) -> io::Result<()> {
        let mut events = Events::default();
        self.selector.try_select(&mut events, None)?;
        for event in events.iter() {
            if let Some(waker) = self.blocked_fds.get(&Token::from_ptr(event.udata as _)) {
                waker.wake_by_ref();
            }
        }
        Ok(())
    }

    /// Tries to register the given `fd` into the `selector` to monitor IO events, which is specified by the
    /// `interest`.
    pub(crate) fn try_register<Fd>(&mut self, fd: &Fd, interest: Interest) -> io::Result<()>
    where
        Fd: os::fd::AsFd + os::fd::AsRawFd,
    {
        self.selector
            .try_register(fd.as_raw_fd(), fd.as_raw_fd().into(), interest)
    }

    /// Tries to deregister the given `fd` from the `selector`.
    pub(crate) fn try_deregister<Fd>(&mut self, fd: &Fd) -> io::Result<()>
    where
        Fd: os::fd::AsFd + os::fd::AsRawFd,
    {
        self.blocked_fds.remove(&fd.as_raw_fd().into());
        self.selector.try_deregister(fd.as_raw_fd())
    }

    /// Blocks when the given `fd` is not ready to use yet and setup the given `waker` to wake up the corresponding
    /// downstream task to poll later.
    pub(crate) fn block<Fd>(&mut self, fd: &Fd, waker: task::Waker)
    where
        Fd: os::fd::AsFd + os::fd::AsRawFd,
    {
        self.blocked_fds.insert(fd.as_raw_fd().into(), waker);
    }
}
