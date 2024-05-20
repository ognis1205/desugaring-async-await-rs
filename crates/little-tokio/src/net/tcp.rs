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

//! This module contains the implementation of TCP related network demultiplexing utilities.

use crate::core::interest::Interest;
use crate::core::runtime::RUNTIME;
use std::{future, io, net, ops, pin, task};

/// Represents the Little Tokio wrapper arround a `TcpListener`. This wrapper is essentially equivalent to
/// `TcpListener`. It implements `Deref` and `DerefMut` to delegate the underlying `TcpListener` methods.
/// Additionally, this struct is responsible for `register` and/or `deregister` (IO demultiplexing) the
/// network IO events to the Little Tokio runtime, which is the core part of this crate.
pub(crate) struct Listener {
    delegatee: net::TcpListener,
}

impl Listener {
    /// Binds inner `TcpListener` to the given `addr` and sets it non-blocking mode.
    pub(crate) fn bind(addr: impl net::ToSocketAddrs) -> io::Result<Self> {
        let delegatee = net::TcpListener::bind(addr)?;
        delegatee.set_nonblocking(true)?;
        Ok(Self { delegatee })
    }

    /// Accepts the incoming connection and returns an `Accept` struct, which offers an abstraction over
    /// IO demultiplexing using the Rust's `Future` runtime, i.e., the Little Tokio runtime.
    pub(crate) fn accept(&mut self) -> impl future::Future<Output = AcceptOutput> + '_ {
        Accept::new(self)
    }
}

impl ops::Deref for Listener {
    type Target = net::TcpListener;

    fn deref(&self) -> &Self::Target {
        &self.delegatee
    }
}

impl ops::DerefMut for Listener {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.delegatee
    }
}

/// Represents the acceptance of a TCP connection, abstracting the IO demultiplexing of the Little Tokio runtime.
/// It provides the following two functionalities:
///  - Registration of the file descriptor to the runtime to monitor readiness for reading from the associated stream.
///  - Implementation of the `Future` trait for the event loop of the runtime to await read-ready events.
pub(crate) struct Accept<'a>(&'a mut Listener);

impl<'a> Accept<'a> {
    /// Creates a new `Accept` instance from the specified `listener` and registers it to the runtime.
    pub(crate) fn new(listener: &'a mut Listener) -> Self {
        listener
            .delegatee
            .set_nonblocking(true)
            .expect("should make the TCP listener non blocking properly");
        RUNTIME.with_borrow_mut(|runtime| {
            runtime
                .as_mut()
                .unwrap()
                .try_register(&listener.delegatee, Interest::READABLE)
                .expect("should make the TCP listener non blocking properly");
        });
        Self(listener)
    }
}

impl ops::Deref for Accept<'_> {
    type Target = Listener;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Accept<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

///
pub(crate) type AcceptOutput = io::Result<(Stream, net::SocketAddr)>;

impl<'a> future::Future for Accept<'a> {
    type Output = AcceptOutput;

    ///
    fn poll(self: pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        match self.delegatee.accept() {
            Ok((stream, addr)) => task::Poll::Ready(Ok((Stream::new(stream)?, addr))),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                RUNTIME.with_borrow_mut(|runtime| {
                    runtime
                        .as_mut()
                        .unwrap()
                        .notify(&self.delegatee, cx.waker().clone())
                });
                task::Poll::Pending
            }
            Err(e) => task::Poll::Ready(Err(e)),
        }
    }
}

///
pub(crate) struct Stream(net::TcpStream);

impl Stream {
    ///
    pub(crate) fn new(stream: net::TcpStream) -> io::Result<Self> {
        todo!()
    }
}

impl ops::Deref for Stream {
    type Target = net::TcpStream;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Stream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
