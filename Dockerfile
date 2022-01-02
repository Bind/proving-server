FROM rust:1.57-slim

WORKDIR /app
COPY . .

RUN set -ex; \ 
  apt-get update; \
  apt-get install -y --no-install-recommends \
  pkg-config \
  libssl-dev \
  sqlite3 libsqlite3-dev

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=6666

RUN cargo build --release

CMD ["cargo", "run"]