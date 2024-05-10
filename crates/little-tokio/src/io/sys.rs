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

//! This module contains the implementation of OS specific bindings.

#[cfg(any(target_os = "macos"))]
pub(crate) mod unix;

// Wraps system call bindings to transform system call return values into Rust's `Result`.
#[allow(unused_macros)]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let ret = unsafe { libc::$fn($($arg, )*) };
        if ret < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(ret)
        }
    }};
}
