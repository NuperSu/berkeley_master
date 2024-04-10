# Stage 1: Build the application
FROM rust:1.77 as builder

# Create a new empty shell project
RUN USER=root cargo new --bin berkeley_master
WORKDIR /berkeley_master

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# This trick will cache our dependencies
RUN cargo build --release
RUN rm src/*.rs

# Now that we've got a dummy project built and our dependencies cached,
# let's copy our source code in and build the real project.
COPY ./src ./src

RUN rm ./target/release/deps/berkeley_master*
RUN cargo build --release

# Stage 2: Setup the runtime environment
FROM debian:bookworm-slim

# Copy the binary from the builder stage
COPY --from=builder /berkeley_master/target/release/berkeley_master /usr/local/bin

# Set the default command of the container to run your application
CMD ["sh", "-c", "berkeley_master $ARGS"]
