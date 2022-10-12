## Packet Monitoring built for CSCI551

### Pre-requisites

To build this project, you need to have the following installed:

- Rust - can be acquired using the following command:

```bash
curl --proto "=https" --tlsv1.2 --retry 3 -sSfL https://sh.rustup.rs | sh -s -- -y
```

- Cargo - Can be acquired using the same command as above
- cc - can be acquired using the following command:

```bash
sudo apt install -y build-essential
```

To do an offline install as required on DETER, please follow the following instructions:
[https://hackmd.io/@dpjungmin/install-rust-in-deter](https://hackmd.io/@dpjungmin/install-rust-in-deter)

### Build

There are three ways to build the packet monitor.
If you are using an Ubuntu 16.04 build machine you can go with methods 1 or 2.

#### Build with Makefile (Recommended)

```sh
make
```

This generates a binary called `proja` in the current directory.
It also generates a binary at `target/release/packet_monitor`.

#### Build with Cargo

```sh
cargo build --release
```

It generates a binary at `target/release/packet_monitor`.

#### Build with cross-rs

This method is recommended when you are using a machine with a different version of
Ubuntu than the one used in the lab machines.
It relies on Docker and [cross-rs](https://github.com/cross-rs/cross) to build the binary.

To install the tools run the following commands:

```sh
sudo apt-get install -y docker
cargo install cross
```

To build the binary run the following command:

```sh
cross build --target x86_64-unknown-linux-gnu --release
```

This puts the built binary in `target/x86_64-unknown-linux-gnu/release/`.

### Usage

```bash
./proja -d drop-packet <INTERFACE>
```

This program HAS to be run with root privileges (since it may modify iptables or /etc/quagga/bgpd.conf).

For detailed usage, please use

```bash
./proja --help
```

The data tsv file that is generated is called `dump.tsv`
The extreme events file is called `events.tsv`. These events are also printed to stdout.

### External Dependencies

The project uses the two following external dependencies:

- clap - Used for argument parsing
- tokio - Async runtime with async versions of the standard library
