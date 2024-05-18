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

//! This module contains the implementation of a `Token` which represents the user defined `udata`
//! of the `kevent` system call.

use crate::core::task::Id as TaskId;
use std::fmt;

/// Identifies a file descriptor to track which data source generated the event.
#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Token(usize);

impl Token {
    /// Returns the copy of the current `Token` and increments the internal `usize` value.
    pub(crate) fn increment(&mut self) -> Self {
        let ret = Self(self.0);
        self.0 += 1;
        ret
    }

    /// According to the document [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html),
    /// the `udata` field in `kevent` is an opaque user defined data field which can be utilized by
    /// the user. We use this field for `Token` to identify the event source.
    pub(crate) fn to_ptr(self) -> *const () {
        self.0 as _
    }

    /// According to the document [kevent(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kevent.2.html),
    /// the `udata` field in `kevent` is an opaque user defined data field which can be utilized by
    /// the user. We use this field for `Token` to identify the event source.
    pub(crate) fn from_ptr(value: *const ()) -> Self {
        Self(value as _)
    }
}

impl From<TaskId> for Token {
    fn from(id: TaskId) -> Self {
        Self::from_ptr(id.to_ptr())
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.0)?;
        Ok(())
    }
}
