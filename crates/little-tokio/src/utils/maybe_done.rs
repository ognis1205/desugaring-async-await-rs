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

//! This module contains the implementation of `MaybeDone` combinator.

use crate::utils::misc::assert_future;
use std::{future, pin, task};

/// Represents a `Future` that may have done.
pub enum MaybeDone<F>
where
    F: future::Future,
{
    Future(/* pinned */ F),
    Done(F::Output),
    Gone,
}

impl<F> Unpin for MaybeDone<F> where F: future::Future + Unpin {}

impl<Fut> future::Future for MaybeDone<Fut>
where
    Fut: future::Future,
{
    type Output = ();

    //    fn poll(mut self: pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
    fn poll(mut self: pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        // Safety:
        unsafe {
            match self.as_mut().get_unchecked_mut() {
                Self::Future(f) => {
                    let done = ready!(pin::Pin::new_unchecked(f).poll(cx));
                    self.set(Self::Done(done));
                }
                Self::Done(_) => {}
                Self::Gone => panic!("MaybeDone polled after value taken"),
            }
        }
        task::Poll::Ready(())
    }
}

/// Wraps a `Future` into a `MaybeDone`.
pub fn maybe_done<F>(future: F) -> MaybeDone<F>
where
    F: future::Future,
{
    assert_future::<(), _>(MaybeDone::Future(future))
}
