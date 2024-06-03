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

//! This module contains utility combinators of `Future`.

// Bakes in propagation of `Pending` signals by returning early.
#[allow(unused_macros)]
macro_rules! ready {
    ($poll: expr $(,)?) => {
        match $poll {
            std::task::Poll::Ready(output) => output,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}

pub mod maybe_done;
pub(crate) mod misc;
