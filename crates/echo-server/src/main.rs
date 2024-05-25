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

use clap::Parser;
use little_tokio::{
    self,
    net::tcp::{Listener as TcpListener, Stream as TcpStream},
};

#[derive(Parser, Debug)]
#[command(version, about, author, long_about = None)]
struct Cli {
    #[arg(short = 'p', long, default_value_t = 5000, help = "Port number")]
    port: u16,
}

fn main() {
    let args = Cli::parse();
    little_tokio::block_on(async move {
        let mut listener = TcpListener::bind(format!("0.0.0.0:{}", args.port)).unwrap();
        println!("server listening on: {}", listener.local_addr().unwrap());
        loop {
            let (connection, _) = listener.accept().await.unwrap();
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
        connection.write(&buffer[..count]).await.unwrap();
    }
}
