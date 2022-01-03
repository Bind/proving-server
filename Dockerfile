FROM rustlang/rust:nightly-slim AS chef
RUN cargo install cargo-chef 
WORKDIR app
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN set -ex; \ 
  apt-get update; \
  apt-get install -y --no-install-recommends \
  pkg-config \
  libssl-dev \
  sqlite3 libsqlite3-dev
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .

RUN cargo build --release --bin proving-server

# We do not need the Rust toolchain to run the binary!
FROM debian:bullseye-slim AS runtime
WORKDIR app

RUN set -ex; \ 
  apt-get update; \
  apt-get install -y --no-install-recommends \
  pkg-config \
  sqlite3
COPY --from=builder /app/target/release/proving-server /usr/local/bin
ENV ZK_FILE_PATH="/app/zk_files/"
EXPOSE $PORT
CMD ROCKET_PORT=$PORT /usr/local/bin/proving-server

