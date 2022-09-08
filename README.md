## Packet Monitoring built for CSCI551

### Build

> **Note**
> This project uses [cross-rs](https://github.com/cross-rs/cross). Please install it first.

```bash
cross build --target x86_64-unknown-linux-gnu --release
```

This puts the built binary in `target/x86_64-unknown-linux-gnu/release/`.

Cross is used so that the required version of glibc can be used.

### Run

```bash
./target/x86_64-unknown-linux-gnu/release/packet-monitor
```

This runs the binary.
