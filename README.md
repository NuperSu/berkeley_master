# Berkeley Master Node

The `berkeley_master` program is a Rust-based implementation of the master node for the Berkeley Clock Synchronization
Algorithm. It's designed to synchronize the system time across multiple slave nodes in a distributed system.

## Features

- Initiates time synchronization among slave nodes.
- Periodically calculates the average time difference and sends adjustment instructions to each slave node.
- Uses UDP for network communication.

## Requirements

To compile and run this program, you will need:

- Rust programming environment (see [the Rust installation guide](https://www.rust-lang.org/tools/install))
- Cargo (comes with Rust)

## Compilation

Navigate to the root directory of the `berkeley_master` project and run the following command to compile the program:

```bash
cargo build --release
```

This command generates an executable in the `target/release` directory.

## Running the Master Node

To run the master node, use the following command syntax:

```bash
cargo run -- [Master Node Address] [Slave Node Addresses...]
```

- `[Master Node Address]` is the IP address and port that the master node binds to (e.g., `127.0.0.1:8080`).
- `[Slave Node Addresses...]` is a space-separated list of IP addresses and ports for each slave node you wish to
  synchronize (e.g., `127.0.0.1:8081 127.0.0.1:8082`).

### Example

```bash
cargo run -- 127.0.0.1:8080 127.0.0.1:8081 127.0.0.1:8082
```

This command starts the master node on `127.0.0.1:8080` and attempts to synchronize time with slave nodes
on `127.0.0.1:8081` and `127.0.0.1:8082`.

### Docker Support

The `berkeley_master` application can also be run as a Docker container, which simplifies deployment and ensures
consistent runtime environments.

#### Building the Docker Image

First, ensure Docker is installed on your system. Then, build the Docker image for berkeley_master by navigating to the
root directory of the project and running:

```bash
docker build -t berkeley_master .
```

This command builds a Docker image named berkeley_master based on the Dockerfile in the current directory.

#### Running the Docker Container

After building the image, run the master node within a Docker container using:

```bash
docker run --net=host -e ARGS="[Master Node Address] [Slave Node Addresses...]" berkeley_master
```

- `[Master Node Address]` is the IP address and port that the master node binds to (e.g., `127.0.0.1:8080`).
- `[Slave Node Addresses...]` is a space-separated list of IP addresses and ports for each slave node you wish to
  synchronize (e.g., `127.0.0.1:8081 127.0.0.1:8082`).

#### Example

To start the master node on 127.0.0.1:8080 and synchronize time with slave nodes on 127.0.0.1:8081 and 127.0.0.1:8082:

```bash
docker run --net=host -e ARGS="127.0.0.1:8080 127.0.0.1:8081 127.0.0.1:8082" berkeley_master
```
