# desugaring-async-await-rs

This repository hosts a Rust implementation of [`Future`](https://doc.rust-lang.org/std/future/trait.Future.html)
runtime for educational purposes. It comprises two main crates:

 - [little-tokio](./crates/little-tokio)
 - [echo-server](./crates/echo-server)

The [little-tokio](./crates/little-tokio) crate offers a minimal implementation of Rust's
[`Future`](https://doc.rust-lang.org/std/future/trait.Future.html) runtime library, specifically tailored for MacOSX and
built upon BSD's [`kqueue(2)`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kqueue.2.html)
mechanism. While the core design is influenced by [tokio-rs/mio](https://github.com/tokio-rs/mio),
it's intentionally simplified for learning purposes.

The [echo-server](./crates/echo-server) crate offers the example implementation of an echo server based on
[little-tokio](./crates/little-tokio) crate.

## What You Can Expect to Learn

 - [IO MUX/DEMUX](https://en.wikipedia.org/wiki/Multiplexing) ([Reactor Pattern](https://en.wikipedia.org/wiki/Reactor_pattern))
 - Rust [async/.await](https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html) tools and [Future](https://doc.rust-lang.org/std/future/trait.Future.html) trait as well as [Pin](https://doc.rust-lang.org/std/pin/struct.Pin.html) trait and [Pin projection](https://doc.rust-lang.org/std/pin/index.html#projections-and-structural-pinning)
 - Rust C/C++ [FFI](https://en.wikipedia.org/wiki/Foreign_function_interface)

## Links
