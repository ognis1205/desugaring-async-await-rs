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

//! This module contains the implementation of UNIX `kqueue` bindings.

/// Represents the Rust wrapper arround a libc `kevent`.
///
/// # See also:
/// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
pub(crate) type Event = libc::kevent;

/// Represents the Rust wrapper around a libc `kevent`. This wrapper is essentially equivalent to
/// Rust's `Vec` and consists of `kevent` elements. It implements `Deref` and `DerefMut` to delegate
/// the underlying `Vec` methods.
///
/// # See also:
/// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
pub(crate) struct Events(Vec<libc::kevent>);

impl Events {
    /// Creates `Events` with a given `capacity`.
    pub fn with_capacity(capacity: usize) -> Events {
        Events(Vec::with_capacity(capacity))
    }
}

impl Deref for Events {
    type Target = Vec<libc::kevent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Events {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// This module provides `kqueue`-related utility functions.
pub(crate) mod event {
    /// Returns `true` if the `kevent` representing there is data available to read.
    pub(crate) fn is_readable(event: &Event) -> bool {
        event.filter == libc::EVFILT_READ || event.filter == libc::EVFILT_USER
    }

    /// Returns `true` if the `kevent` representing it is possible to write to the associated file
    /// descriptor.
    pub(crate) fn is_writable(event: &Event) -> bool {
        event.filter == libc::EVFILT_WRITE
    }

    /// Returns `true` if an error occurs while processing an element of the `changes`.
    pub(crate) fn is_error(event: &Event) -> bool {
        (event.flags & libc::EV_ERROR) != 0
            || (event.flags & libc::EV_EOF) != 0 && event.fflags != 0
    }

    /// Returns `true` if the `kevent` is waiting for a reading event and the associated data is closed
    /// before it reaches to the EOF.
    pub(crate) fn is_read_closed(event: &Event) -> bool {
        event.filter == libc::EVFILT_READ && event.flags & libc::EV_EOF != 0
    }

    /// Returns `true` if the `kevent` is waiting for a writing event and the associated data is closed
    /// before it reaches to the EOF.
    pub(crate) fn is_write_closed(event: &Event) -> bool {
        event.filter == libc::EVFILT_WRITE && event.flags & libc::EV_EOF != 0
    }
}
