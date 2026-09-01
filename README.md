# nremote-server

The rendezvous and relay services behind [nremote](https://github.com/NDDev-OpenNetwork/nremote),
a self-hosted remote-desktop system.

Two binaries:

| Binary | Role | Listens on |
| --- | --- | --- |
| `hbbs` | rendezvous — registers device IDs, brokers direct connections | `21115/tcp`, `21116/tcp`, `21116/udp`, `21118/tcp` (websocket) |
| `hbbr` | relay — carries the session when a direct connection cannot be made | `21117/tcp`, `21119/tcp` (websocket) |

A third binary, `nremote-utils`, generates and validates key pairs and checks a
server from the outside.

Most sessions never touch the relay: `hbbs` hole-punches and the two peers talk
directly. The relay exists for the pairs that cannot, which on real networks is
a minority but never zero.

## Run it

```bash
mkdir -p data && chown 10001:10001 data   # the image runs as uid 10001
docker compose up -d
```

Then read the public key the server generated on first start:

```bash
cat data/id_ed25519.pub
```

That key and the server's hostname are the two values a client needs. Nothing
else is required, and neither value is a secret — every client that connects has
to know both.

`docker-compose.yml` in this repository is an example with a placeholder host.
Set `-r` to your own address before using it.

## Ports, and the two you should leave closed

Open `21115/tcp`, `21116/tcp`, `21116/udp` and `21117/tcp`. UDP is not optional:
`21116/udp` carries ID registration and the heartbeat, and a firewall that
allows only the TCP half produces a server that accepts connections and never
registers anyone.

Leave `21118` and `21119` closed unless a reverse proxy sits in front of them.
Both services read `X-Real-IP` and `X-Forwarded-For` on those websocket
listeners and trust what they say. Anything that can reach them directly can
claim any source address, which defeats per-IP rate limiting and corrupts every
address in the log. They are useful behind a proxy that overwrites both headers.
Directly exposed, they are a hole.

## Keys

`hbbs` generates `id_ed25519` and `id_ed25519.pub` in its working directory on
first start. Keep the private half; losing it means every client has to be
reconfigured with the new public key.

`hbbr` defaults to an **empty** key, and empty does not mean "generate one" — it
means relay validation is switched off, so anyone who can reach `21117` can push
traffic through the relay. Pass `-k _` to make it load the same pair `hbbs`
created. The example compose file does.

## Configuration

Flags, environment variables and the `.env` / `--config` precedence rules are
documented in [docs/environment-variables.md](docs/environment-variables.md).
The four that matter most:

| Setting | Flag | Applies to | Purpose |
| --- | --- | --- | --- |
| Key | `-k` | both | `_` loads or generates the key pair |
| Relay address | `-r` | `hbbs` | what clients are told to relay through |
| Port | `-p` | both | `hbbs` also binds `PORT-1` and `PORT+2` |
| Force relay | `ALWAYS_USE_RELAY=Y` | `hbbs` | disables direct connections entirely |

## Build

```bash
cargo build --release --locked
```

Needs a Rust toolchain and OpenSSL development headers (`libssl-dev` on Debian
and Ubuntu). The container build in `Dockerfile` is the reference: if a change
builds there, it builds.

## Licence

AGPL-3.0-or-later. See [LICENSE](LICENSE), and [NOTICE](NOTICE) for the
attribution and the list of modifications this repository carries.
