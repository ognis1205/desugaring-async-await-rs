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
use crate::core::reactor::Reactor;
use pin_project::{pin_project, pinned_drop};
use std::io::Read as _;
use std::io::Write as _;
use std::{future, io, net, ops, pin, task};

/// Represents the Little Tokio wrapper arround a `TcpListener`. This wrapper is essentially equivalent to
/// `TcpListener`. It implements `Deref` and `DerefMut` to delegate the underlying `TcpListener` methods.
/// Additionally, this struct is responsible for `register` and/or `deregister` (IO demultiplexing) the
/// network IO events to the Little Tokio runtime, which is the core part of this crate.
pub struct Listener {
    delegatee: net::TcpListener,
}

impl Listener {
    /// Binds inner `TcpListener` to the given `addr` and sets it non-blocking mode.
    pub fn bind(addr: impl net::ToSocketAddrs) -> io::Result<Self> {
        let delegatee = net::TcpListener::bind(addr)?;
        delegatee.set_nonblocking(true)?;
        Ok(Self { delegatee })
    }

    /// Accepts the incoming connection and returns an `Accept` struct, which offers an abstraction over
    /// IO demultiplexing using the Rust's `Future` runtime, i.e., the Little Tokio runtime.
    pub fn accept(&mut self) -> impl future::Future<Output = AcceptOutput> + '_ {
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
pub struct Accept<'listener> {
    listener: &'listener mut Listener,
}

impl<'listener> Accept<'listener> {
    /// Creates a new `Accept` instance from the specified `listener` and registers it to the runtime.
    fn new(listener: &'listener mut Listener) -> Self {
        listener
            .delegatee
            .set_nonblocking(true)
            .expect("should make the TCP listener non blocking properly");
        Reactor::register(&listener.delegatee, Interest::READABLE);
        Self { listener }
    }
}

pub type AcceptOutput = io::Result<(Stream, net::SocketAddr)>;

impl<'listener> future::Future for Accept<'listener> {
    type Output = AcceptOutput;

    fn poll(self: pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        match self.listener.delegatee.accept() {
            Ok((stream, addr)) => task::Poll::Ready(Ok((Stream::new(stream)?, addr))),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                Reactor::block(&self.listener.delegatee, cx.waker().clone());
                task::Poll::Pending
            }
            Err(e) => task::Poll::Ready(Err(e)),
        }
    }
}

impl<'listener> Drop for Accept<'listener> {
    fn drop(&mut self) {
        Reactor::deregister(&self.listener.delegatee);
    }
}

/// Represents the Little Tokio wrapper arround a `TcpStream`. This wrapper is essentially equivalent to
/// `TcpStream`. It implements `Deref` and `DerefMut` to delegate the underlying `TcpStream` methods.
/// Additionally, this struct is responsible for `register` and/or `deregister` (IO demultiplexing) the
/// network IO events to the Little Tokio runtime, which is the core part of this crate.
pub struct Stream {
    delegatee: net::TcpStream,
}

impl Stream {
    /// Creates a new `Stream` instance from the specified `stream` and sets it non-blocking mode.
    fn new(stream: net::TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        Ok(Self { delegatee: stream })
    }

    /// Reads from the incoming connection and returns an `Read` struct, which offers an abstraction over
    /// IO demultiplexing using the Rust's `Future` runtime, i.e., the Little Tokio runtime.
    pub fn read<'stream, 'buffer>(
        &'stream mut self,
        buffer: &'buffer mut [u8],
    ) -> impl future::Future<Output = ReadOutput> + 'stream
    where
        'buffer: 'stream,
    {
        Read::new(self, buffer)
    }

    /// Writes to the outgoing connection and returns an `Write` struct, which offers an abstraction over
    /// IO demultiplexing using the Rust's `Future` runtime, i.e., the Little Tokio runtime.
    pub fn write<'stream, 'buffer>(
        &'stream mut self,
        buffer: &'buffer [u8],
    ) -> impl future::Future<Output = WriteOutput> + 'stream
    where
        'buffer: 'stream,
    {
        Write::new(self, buffer)
    }
}

impl ops::Deref for Stream {
    type Target = net::TcpStream;

    fn deref(&self) -> &Self::Target {
        &self.delegatee
    }
}

impl ops::DerefMut for Stream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.delegatee
    }
}

/// Represents the read event of a TCP connection, abstracting the IO demultiplexing of the Little Tokio runtime.
/// It provides the following two functionalities:
///  - Registration of the file descriptor to the runtime to monitor readiness for reading from the associated stream.
///  - Implementation of the `Future` trait for the event loop of the runtime to await read-ready events.
#[pin_project(PinnedDrop)]
struct Read<'stream, 'buffer> {
    stream: &'stream mut Stream,
    buffer: &'buffer mut [u8],
}

impl<'stream, 'buffer> Read<'stream, 'buffer> {
    /// Creates a new `Read` instance from the specified `stream` and registers it to the runtime.
    fn new(stream: &'stream mut Stream, buffer: &'buffer mut [u8]) -> Self {
        stream
            .delegatee
            .set_nonblocking(true)
            .expect("should set non-blocking properly");
        Reactor::register(&stream.delegatee, Interest::READABLE);
        Self { stream, buffer }
    }
}

pub type ReadOutput = io::Result<usize>;

impl<'stream, 'buffer> future::Future for Read<'stream, 'buffer> {
    type Output = ReadOutput;

    fn poll(self: pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let this = self.project();
        let stream = &mut this.stream.delegatee;
        let buffer = this.buffer;
        match stream.read(buffer) {
            Ok(size) => task::Poll::Ready(Ok(size)),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                Reactor::block(stream, cx.waker().clone());
                task::Poll::Pending
            }
            Err(e) => task::Poll::Ready(Err(e)),
        }
    }
}

#[pinned_drop]
impl<'stream, 'buffer> PinnedDrop for Read<'stream, 'buffer> {
    fn drop(self: pin::Pin<&mut Self>) {
        Reactor::deregister(&self.stream.delegatee);
    }
}

/// Represents the write event of a TCP connection, abstracting the IO demultiplexing of the Little Tokio runtime.
/// It provides the following two functionalities:
///  - Registration of the file descriptor to the runtime to monitor readiness for writing to the associated stream.
///  - Implementation of the `Future` trait for the event loop of the runtime to await read-ready events.
#[pin_project(PinnedDrop)]
struct Write<'stream, 'buffer> {
    stream: &'stream mut Stream,
    buffer: &'buffer [u8],
}

impl<'stream, 'buffer> Write<'stream, 'buffer> {
    /// Creates a new `Write` instance from the specified `stream` and registers it to the runtime.
    fn new(stream: &'stream mut Stream, buffer: &'buffer [u8]) -> Self {
        stream
            .delegatee
            .set_nonblocking(true)
            .expect("should set non-blocking properly");
        Reactor::register(&stream.delegatee, Interest::WRITABLE);
        Self { stream, buffer }
    }
}

pub type WriteOutput = io::Result<usize>;

impl<'stream, 'buffer> future::Future for Write<'stream, 'buffer> {
    type Output = WriteOutput;

    fn poll(self: pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        //    fn poll(mut self: pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let this = self.project();
        let stream = &mut this.stream.delegatee;
        let buffer = this.buffer;
        //        let this = &mut *self;
        match stream.write(buffer) {
            //        match this.stream.delegatee.write(this.buffer) {
            Ok(size) => task::Poll::Ready(Ok(size)),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                Reactor::block(stream, cx.waker().clone());
                //                Reactor::block(&this.stream.delegatee, cx.waker().clone());
                task::Poll::Pending
            }
            Err(e) => task::Poll::Ready(Err(e)),
        }
    }
}

#[pinned_drop]
impl<'stream, 'buffer> PinnedDrop for Write<'stream, 'buffer> {
    fn drop(self: pin::Pin<&mut Self>) {
        Reactor::deregister(&self.stream.delegatee);
    }
}
