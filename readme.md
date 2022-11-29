# TCP Proxy

The structure for config is in the `config.json` file.

For each app, you can have a list of ports and a list of targets.

Using tokio a task is spawned for each port to listen on and inside each spawned task 
for each incoming a request a `handle_request` task is spawned.


## Usage

```bash
cargo run --release
```

## To test
 You will need a tcp client.
 
