# TCP Proxy

The structure for config is in the `config.json` file.

For each app, you can have a list of ports and a list of targets.

Using tokio, I spawn a task for each port and targets.

Ideally targets part would be share between all ports, but I thought this way would be more extensible for other usecases.

## Usage

```bash
cargo run --release
```

## To test
 You will need a tcp client.
 