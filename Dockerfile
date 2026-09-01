# syntax=docker/dockerfile:1

# One image carries both services; the command selects which one runs. They
# share a data directory because they share a key pair, and splitting them into
# two images would duplicate a 40 MB build to change one argument.

# Pinned by digest, not by tag alone. A tag is a moving pointer, and this image
# terminates connections from the public internet: the bytes that go into it
# should be the bytes that were reviewed.
FROM rust:1.97-trixie@sha256:b1b3c9c0d921d7fa0a6d1f9ec7e4eab87f8c8ec97644c3d791450f131dec813f AS build

# openssl-sys is reached through tokio-native-tls in the shared library, so the
# development headers are a build input even though nothing here speaks TLS on
# its own behalf. protobuf codegen is pure Rust and needs no protoc.
RUN apt-get update \
 && apt-get install --no-install-recommends --yes pkg-config libssl-dev \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . .

# --locked, because a release that resolves its own dependency versions is not
# the release that was tested. DATABASE_URL is read here by sqlx at compile
# time to check the queries against db_v2.sqlite3; it is not the runtime
# database and has no effect on the running server.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked \
 && mkdir -p /out \
 && cp target/release/hbbs target/release/hbbr target/release/nremote-utils /out/

FROM debian:trixie-slim@sha256:d7e12182ce18b85b93007c1dedf31f2d29e01ccf3182cc4017c709b6259bc132

RUN apt-get update \
 && apt-get install --no-install-recommends --yes ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --user-group --home-dir /data --no-create-home nremote

COPY --from=build /out/hbbs /usr/local/bin/hbbs
COPY --from=build /out/hbbr /usr/local/bin/hbbr
COPY --from=build /out/nremote-utils /usr/local/bin/nremote-utils
COPY --chmod=0755 docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

# Both services write the key pair and the SQLite database into the working
# directory, so the working directory is the state and everything else in this
# image is immutable.
RUN install -d -o 10001 -g 10001 -m 0700 /data
WORKDIR /data
VOLUME /data
USER 10001:10001

# 21115 tcp   hbbs, NAT type test
# 21116 tcp   hbbs, rendezvous
# 21116 udp   hbbs, heartbeat and ID registration
# 21117 tcp   hbbr, relay
# 21118 tcp   hbbs, websocket -- see the README before exposing it
# 21119 tcp   hbbr, websocket relay -- likewise
EXPOSE 21115/tcp 21116/tcp 21116/udp 21117/tcp 21118/tcp 21119/tcp

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]

# No default command. `hbbs` and `hbbr` are different services with different
# arguments, and a default would make one of them the accidental answer.
