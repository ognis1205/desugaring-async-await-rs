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

//! This module contains the implementation of a `Interest` which represents the interest of
//! in either `Read` or `Write` events.

use std::{fmt, num, ops};

/// Represents interest in either Read or Write events. This struct is created by using one of
/// the two constants:
///
/// - Interest::READABLE
/// - Interest::WRITABLE
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Interest(num::NonZeroU8);

const READABLE: u8 = 0b0001;

const WRITABLE: u8 = 0b0010;

impl Interest {
    /// Returns a `Interest` set representing readable interests.
    pub const READABLE: Interest = Interest(unsafe { num::NonZeroU8::new_unchecked(READABLE) });

    /// Returns a `Interest` set representing writable interests.
    pub const WRITABLE: Interest = Interest(unsafe { num::NonZeroU8::new_unchecked(WRITABLE) });

    /// Adds together two `Interest`. This does the same thing as the `BitOr` implementation, but is a
    /// constant function.
    pub const fn add(self, other: Interest) -> Interest {
        Interest(unsafe { num::NonZeroU8::new_unchecked(self.0.get() | other.0.get()) })
    }

    /// Returns true if the value includes readable readiness.
    pub fn is_readable(self) -> bool {
        (self.0.get() & READABLE) != 0
    }

    /// Returns true if the value includes writable readiness.
    pub fn is_writable(self) -> bool {
        (self.0.get() & WRITABLE) != 0
    }
}

impl ops::BitOr for Interest {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self {
        self.add(other)
    }
}

impl ops::BitOrAssign for Interest {
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        self.0 = (*self | other).0;
    }
}

impl fmt::Debug for Interest {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut is_flagged = false;
        if self.is_readable() {
            if is_flagged {
                write!(fmt, " | ")?
            }
            write!(fmt, "READABLE")?;
            is_flagged = true
        }
        if self.is_writable() {
            if is_flagged {
                write!(fmt, " | ")?
            }
            write!(fmt, "WRITABLE")?;
            is_flagged = true
        }
        debug_assert!(is_flagged, "printing empty interests");
        Ok(())
    }
}
