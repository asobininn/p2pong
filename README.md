# p2pong
This is a test program using bevy.
## How to start
Enter the following command.
```rust:p2pong
cargo run -- --local-port 7000 --players localhost 127.0.0.1:7001
cargo run -- --local-port 7001 --players 127.0.0.1:7000 localhost
```