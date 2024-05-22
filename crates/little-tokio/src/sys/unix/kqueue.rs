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

use crate::core::interest::Interest;
use crate::core::token::Token;
use std::{cmp, default, io, mem, ops, os, ptr, slice, time};

/// Represents raw OS error codes returned by system calls.
type RawOsError = i32;

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

// Wraps `libc::kevent` so that the arguments will be coerced as its FFI defined.
macro_rules! new_kevent {
    ($id: expr, $filter: expr, $flags: expr, $udata: expr) => {
        libc::kevent {
            ident: $id as Id,
            filter: $filter as Filter,
            flags: $flags as Flags,
            udata: $udata as UData,
            // Safety:
            // The remaining fields are `fflags` and `data`. These filter-specific fields are utilized by the
	    // kernel and vary depending on the file descriptor types, in other words, these are irrelevant
	    // to the user land so it is safe to fill out with zeros.
            ..unsafe { std::mem::zeroed() }
        }
    };
}

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

impl ops::Deref for Event {
    type Target = libc::kevent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Event {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl default::Default for Event {
    fn default() -> Self {
        Self(new_kevent!(0, 0, 0, 0))
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

impl ops::Deref for Events {
    type Target = Vec<libc::kevent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Events {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl default::Default for Events {
    fn default() -> Self {
        Self(vec![*Event::default()])
    }
}

/// Checks all events for possible errors, it returns the first error found.
///
/// # See also:
/// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
fn check_errors(events: &[libc::kevent], ignored_errors: &[RawOsError]) -> io::Result<()> {
    for event in events {
        // Note:
        // We can't use references to packed structures (in checking the ignored errors), so we
        // need copy the data out before use.
        let data = event.data as _;
        // Note:
        // Check for the error flag, the actual error will be in the `data` field.
        if (event.flags & libc::EV_ERROR != 0) && data != 0 && !ignored_errors.contains(&data) {
            return Err(io::Error::from_raw_os_error(data as RawOsError));
        }
    }
    Ok(())
}

/// Registers `changelist` with `kq`ueue.
///
/// # See also:
/// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
fn register_kevents(
    kq: os::fd::RawFd,
    changelist: &mut [libc::kevent],
    ignored_errors: &[RawOsError],
) -> io::Result<()> {
    syscall!(kevent(
        kq,
        changelist.as_ptr(),
        changelist.len() as Count,
        changelist.as_mut_ptr(),
        changelist.len() as Count,
        ptr::null(),
    ))
    .map(|_| ())
    .or_else(|err| {
        // Note:
        // According to the manual page of FreeBSD: "When `kevent()` call fails with `EINTR` error,
        // all changes in the changelist have been applied", so we can safely ignore it.
        if err.raw_os_error() == Some(libc::EINTR) {
            Ok(())
        } else {
            Err(err)
        }
    })
    .and_then(|()| check_errors(changelist, ignored_errors))
}

/// The MacOSX `kqueue` based IO Mux/Demux.
pub(crate) struct Selector {
    /// Holds the `kqueue` file descriptor.
    pub(crate) kq: os::fd::RawFd,
}

impl Selector {
    /// Tries to create the `kqueue` based IO Mux/Demux.
    pub(crate) fn try_new() -> io::Result<Self> {
        let kq = syscall!(kqueue())?;
        let selector = Self { kq };
        syscall!(fcntl(kq, libc::F_SETFD, libc::FD_CLOEXEC))?;
        Ok(selector)
    }

    /// Tries to select/mux ready `kevents` into `eventlist` with a maximal interval `timeout` to wait for an event.
    ///
    /// # See also:
    /// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
    pub(crate) fn try_select(
        &self,
        eventlist: &mut Events,
        timeout: Option<time::Duration>,
    ) -> io::Result<()> {
        let timeout = timeout.map(|to| libc::timespec {
            tv_sec: cmp::min(to.as_secs(), libc::time_t::MAX as u64) as libc::time_t,
            // Note:
            // `Duration::subsec_nanos` is guaranteed to be less than one billion (the number of
            // nanoseconds in a second), making the cast to `i32` safe. The cast itself is needed for
            // platforms where C's long is only 32 bits.
            tv_nsec: libc::c_long::from(to.subsec_nanos() as i32),
        });
        let timeout = timeout
            .as_ref()
            .map(|s| s as *const _)
            .unwrap_or(ptr::null_mut());
        eventlist.clear();
        syscall!(kevent(
            self.kq,
            ptr::null(),
            0,
            eventlist.as_mut_ptr(),
            eventlist.capacity() as Count,
            timeout,
        ))
        .map(|nevents| {
            // Safety:
            // This is safe because `kevent` ensures that `nevents` are assigned.
            unsafe { eventlist.set_len(nevents as usize) };
        })
    }

    /// Tries to register the given `fd` into `kqueue` to monitor.
    ///
    /// # See also:
    /// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
    pub(crate) fn try_register(
        &self,
        fd: os::fd::RawFd,
        token: Token,
        interest: Interest,
    ) -> io::Result<()> {
        let flags = libc::EV_CLEAR | libc::EV_RECEIPT | libc::EV_ADD;
        let mut changelist: [mem::MaybeUninit<libc::kevent>; 2] =
            [mem::MaybeUninit::uninit(), mem::MaybeUninit::uninit()];
        let mut nchanges = 0;
        if interest.is_writable() {
            let kevent = new_kevent!(fd, libc::EVFILT_WRITE, flags, token.to_ptr());
            changelist[nchanges] = mem::MaybeUninit::new(kevent);
            nchanges += 1;
        }
        if interest.is_readable() {
            let kevent = new_kevent!(fd, libc::EVFILT_READ, flags, token.to_ptr());
            changelist[nchanges] = mem::MaybeUninit::new(kevent);
            nchanges += 1;
        }
        // Safety:
        // This is safe because we ensure that at least `nchanges` are in the array.
        let changelist = unsafe { slice::from_raw_parts_mut(changelist[0].as_mut_ptr(), nchanges) };
        register_kevents(self.kq, changelist, &[libc::EPIPE as RawOsError])
    }

    /// Tries to deregister the given `fd` from `kqueue` to monitor.
    ///
    /// # See also:
    /// [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html)
    pub(crate) fn try_deregister(&self, fd: os::fd::RawFd) -> io::Result<()> {
        let flags = libc::EV_DELETE | libc::EV_RECEIPT;
        let mut changelist: [libc::kevent; 2] = [
            new_kevent!(fd, libc::EVFILT_WRITE, flags, 0),
            new_kevent!(fd, libc::EVFILT_READ, flags, 0),
        ];
        // Note:
        // the ENOENT error when it comes up. The ENOENT error informs us that the filter wasn't
        // there in first place, but we don't really care about that since our goal is to remove it.
        register_kevents(self.kq, &mut changelist, &[libc::ENOENT as RawOsError])
    }
}

impl default::Default for Selector {
    fn default() -> Self {
        Self::try_new().expect("should instanciate kqueue properly")
    }
}

impl Drop for Selector {
    fn drop(&mut self) {
        match syscall!(close(self.kq)) {
            Ok(..) => (),
            Err(e) => panic!("{}", e),
        }
    }
}
