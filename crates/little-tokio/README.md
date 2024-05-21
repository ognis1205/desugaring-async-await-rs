# little-tokio

The crate offers a minimal implementation of Rust's [`Future`](https://doc.rust-lang.org/std/future/trait.Future.html)
runtime library, specifically tailored for MacOSX and built upon BSD's
[`kqueue(2)`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kqueue.2.html)
mechanism. While the core design is influenced by [tokio-rs/mio](https://github.com/tokio-rs/mio),
it's intentionally simplified for learning purposes.
