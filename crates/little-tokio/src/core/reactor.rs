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

//! This module contains the implementation of a single threaded `Future` reactor.

use crate::core::interest::Interest;
use crate::core::token::Token;
use crate::sys::unix::kqueue::Events;
use crate::sys::unix::kqueue::Selector;
use once_cell::sync::Lazy;
use std::{collections, io, os, sync, task};

/// Provides the interface to access a `Reactor` singleton instance. Since the runtime is
/// designed solely for single-threaded environments, all access to the runtime needs to occur
/// via this singleton instance.
struct Singleton;

impl Singleton {
    /// Returns the [`MutexGuard`](https://doc.rust-lang.org/std/sync/struct.MutexGuard.html) of the
    /// `Reactor` singleton instance.
    #[inline(always)]
    fn instance() -> sync::MutexGuard<'static, Reactor> {
        static INSTANCE: Lazy<sync::Mutex<Reactor>> =
            Lazy::new(|| sync::Mutex::new(Reactor::default()));
        INSTANCE
            .lock()
            .expect("`MutexGuard` of the `Reactor` singleton should be locked properly")
    }
}

/// The Little Tokio reactor which is responsible for I/O multiplexing.
#[derive(Default)]
pub(crate) struct Reactor {
    /// Holds the `libc::kqueue` based IO demultiplexer.
    selector: Selector,
    /// Holds the correspondence between blocked file descriptors' tokens and their corresponding wakers, which
    /// the runtime utilizes to wake up tasks.
    blocked_fds: collections::HashMap<Token, task::Waker>,
}

impl Reactor {
    /// Performs one iteration of the I/O event loop.
    ///
    /// # Note:
    /// We should provide a proper error handling here, e.g., implementing a `Turn` structure which is responsible
    /// for recovering, but this is an educational purpose implementation so that conducting over-engineering
    /// was avoided.
    pub(crate) fn turn() {
        Singleton::instance()
            .try_turn()
            .expect("should turn the event loop properly")
    }

    /// Tries to register the given `fd` into the `selector` to monitor IO events, which is specified by the
    /// `interest`.
    ///
    /// # Note:
    /// We should provide a proper error handling here, e.g., implementing a `Registry` structure which is responsible
    /// for recovering, but this is an educational purpose implementation so that conducting over-engineering
    /// was avoided.
    pub(crate) fn register<Fd>(fd: &Fd, interest: Interest)
    where
        Fd: os::fd::AsFd + os::fd::AsRawFd,
    {
        Singleton::instance()
            .try_register(fd, interest)
            .expect("should register the given file descriptor properly")
    }

    /// Tries to deregister the given `fd` from the `selector`.
    ///
    /// # Note:
    /// We should provide a proper error handling here, e.g., implementing a `Registry` structure which is responsible
    /// for recovering, but this is an educational purpose implementation so that conducting over-engineering
    /// was avoided.
    pub(crate) fn deregister<Fd>(fd: &Fd)
    where
        Fd: os::fd::AsFd + os::fd::AsRawFd,
    {
        Singleton::instance()
            .try_deregister(fd)
            .expect("should deregister the given file descriptor properly")
    }

    /// Blocks when the given `fd` is not ready to use yet and setup the given `waker` to wake up the corresponding
    /// downstream task to poll later.
    pub(crate) fn block<Fd>(fd: &Fd, waker: task::Waker)
    where
        Fd: os::fd::AsFd + os::fd::AsRawFd,
    {
        Singleton::instance().do_block(fd, waker);
    }
}

impl Reactor {
    /// Performs one iteration of the I/O event loop.
    ///
    /// # Note:
    /// We should provide a proper error handling here, e.g., implementing a `Turn` structure which is responsible
    /// for recovering, but this is an educational purpose implementation so that conducting over-engineering
    /// was avoided.
    fn try_turn(&mut self) -> io::Result<()> {
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
    ///
    /// # Note:
    /// We should provide a proper error handling here, e.g., implementing a `Registry` structure which is responsible
    /// for recovering, but this is an educational purpose implementation so that conducting over-engineering
    /// was avoided.
    fn try_register<Fd>(&mut self, fd: &Fd, interest: Interest) -> io::Result<()>
    where
        Fd: os::fd::AsFd + os::fd::AsRawFd,
    {
        self.selector
            .try_register(fd.as_raw_fd(), fd.as_raw_fd().into(), interest)
    }

    /// Tries to deregister the given `fd` from the `selector`.
    ///
    /// # Note:
    /// We should provide a proper error handling here, e.g., implementing a `Registry` structure which is responsible
    /// for recovering, but this is an educational purpose implementation so that conducting over-engineering
    /// was avoided.
    fn try_deregister<Fd>(&mut self, fd: &Fd) -> io::Result<()>
    where
        Fd: os::fd::AsFd + os::fd::AsRawFd,
    {
        self.blocked_fds.remove(&fd.as_raw_fd().into());
        self.selector.try_deregister(fd.as_raw_fd())
    }

    /// Blocks when the given `fd` is not ready to use yet and setup the given `waker` to wake up the corresponding
    /// downstream task to poll later.
    fn do_block<Fd>(&mut self, fd: &Fd, waker: task::Waker)
    where
        Fd: os::fd::AsFd + os::fd::AsRawFd,
    {
        self.blocked_fds.insert(fd.as_raw_fd().into(), waker);
    }
}
