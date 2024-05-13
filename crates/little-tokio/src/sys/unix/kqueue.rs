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

use std::io;
use std::mem::{self, MaybeUninit};
use std::ops::{Deref, DerefMut};

/// Represents the Rust wrapper arround a libc `kevent`.  This wrapper is essentially equivalent to
/// `libc::kevent`. It implements `Deref` and `DerefMut` to delegate the underlying `Vec` methods.
///
/// # See also:
/// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
pub(crate) struct Event(libc::kevent);

impl Event {
    /// Returns `true` if the `kevent` representing there is data available to read.
    pub(crate) fn is_readable(&self) -> bool {
        self.filter == libc::EVFILT_READ || self.filter == libc::EVFILT_USER
    }

    /// Returns `true` if the `kevent` representing it is possible to write to the associated file
    /// descriptor.
    pub(crate) fn is_writable(&self) -> bool {
        self.filter == libc::EVFILT_WRITE
    }

    /// Returns `true` if an error occurs while processing an element of the `changes`.
    pub(crate) fn is_error(&self) -> bool {
        (self.flags & libc::EV_ERROR) != 0 || (self.flags & libc::EV_EOF) != 0 && self.fflags != 0
    }

    /// Returns `true` if the `kevent` is waiting for a reading event and the associated data is closed
    /// before it reaches to the EOF.
    pub(crate) fn is_read_closed(&self) -> bool {
        self.filter == libc::EVFILT_READ && self.flags & libc::EV_EOF != 0
    }

    /// Returns `true` if the `kevent` is waiting for a writing event and the associated data is closed
    /// before it reaches to the EOF.
    pub(crate) fn is_write_closed(&self) -> bool {
        self.filter == libc::EVFILT_WRITE && self.flags & libc::EV_EOF != 0
    }
}

impl Deref for Event {
    type Target = libc::kevent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Event {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Represents the Rust wrapper around a libc `kevent`. This wrapper is essentially equivalent to
/// Rust's `Vec` and consists of `kevent` elements. It implements `Deref` and `DerefMut` to delegate
/// the underlying `Vec` methods.
///
/// # See also:
/// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
pub(crate) struct Events(Vec<libc::kevent>);

impl Events {
    /// Creates `Events` with a given `capacity`.
    pub(crate) fn with_capacity(capacity: usize) -> Events {
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

/// Represents `kevent` id.
///
/// # See also:
/// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
type Id = libc::uintptr_t;

/// Represents the number of `kevent`s.
///
/// # See also:
/// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
type Count = libc::c_int;

/// Represents `kevent` filter.
///
/// # See also:
/// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
type Filter = i16;

/// Represents `kevent` flags.
///
/// # See also:
/// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
type Flags = u16;

/// Represents `kevent` data.
///
/// # See also:
/// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
type UData = *mut libc::c_void;

macro_rules! kevent {
    ($id: expr, $filter: expr, $flags: expr, $data: expr) => {
        libc::kevent {
            ident: $id as Id,
            filter: $filter as Filter,
            flags: $flags as Flags,
            udata: $data as UData,
            // Safety:
            // The remaining fields are opaque user defined ones so it should be okay to zero-filled.
            ..unsafe { mem::zeroed() }
        }
    };
}

/// Checks all events for possible errors, it returns the first error found.
fn check_errors(events: &[libc::kevent], ignored_errors: &[i64]) -> io::Result<()> {
    for event in events {
        // We can't use references to packed structures (in checking the ignored errors), so we need
        // copy the data out before use.
        let data = event.data as _;
        // Check for the error flag, the actual error will be in the `data` field.
        if (event.flags & libc::EV_ERROR != 0) && data != 0 && !ignored_errors.contains(&data) {
            return Err(io::Error::from_raw_os_error(data as i32));
        }
    }
    Ok(())
}
