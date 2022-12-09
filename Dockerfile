FROM rust

# Super useful when things are acting weird
RUN apt update && apt install -y gdb

RUN useradd --uid 1000 advent

WORKDIR /build

# Cache dependencies
COPY Cargo.toml Cargo.toml
COPY src/lib.rs src/lib.rs
RUN cargo build
