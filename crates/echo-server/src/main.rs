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

//! This file contains a minimal implementation of a TCP echo server based on the single threaded IO demultiplexer,
//! i.e., event looping, offered by the Little Tokio async/.await runtime.

use little_tokio::{
    self,
    net::tcp::{Listener as TcpListener, Stream as TcpStream},
};

fn main() {
    little_tokio::block_on(async {
        let mut listener = TcpListener::bind("0.0.0.0:5000")
            .expect("should bind a TCP listener to the given IP properly");
        println!(
            "server listening on: {}",
            listener
                .local_addr()
                .expect("should acquire IP address properly")
        );
        loop {
            let (connection, _) = listener
                .accept()
                .await
                .expect("should accept the TCP request properly");
            little_tokio::spawn(handle(connection));
        }
    });
}

async fn handle(mut connection: TcpStream) {
    let mut buffer = [0u8; 1024];
    while let Ok(count) = connection.read(&mut buffer).await {
        if count == 0 {
            return;
        }
        connection
            .write(&buffer[..count])
            .await
            .expect("should echo back the request body ");
    }
}
