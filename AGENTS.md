# AGENTS.md — nremote-server

## What this is

The rendezvous (`hbbs`) and relay (`hbbr`) services for nremote. Public,
AGPL-3.0, part of `NDDev-OpenNetwork`. `NOTICE` records the prior work this is
derived from and every modification made since; add to that list when you make
one that belongs there.

## The boundary that matters most

**This repository is public and carries no private fact.** No hostname, IP
address, key, account, server name or tenant belongs in it. Examples use
`remote.example.com`. The address of any real deployment, the public key it
generated and the host it runs on live in the private estate that deploys it,
never here. A commit that adds a real address to this repository is a leak, not
a convenience.

## The wire protocol is a contract

Clients in the field speak it. Message definitions in
`libs/hbb_common/protos/` and the field names and values inside them are not
refactorable: renaming one produces a server that starts, listens, and silently
fails to register a device. Comments and identity strings around them are free
to change; the bytes on the wire are not.

## libs/hbb_common is vendored, not a dependency

It arrived as a submodule and is now part of this tree. It contains a good deal
of client-side machinery this server never calls — file transfer, keyboard
mapping, platform probes. Do not delete it on the grounds that it is unused
without checking what the client repository needs from the same file; the two
are meant to stay readable against each other.

Upstream changes to that directory are merged by hand. There is no gitlink to
bump.

## Rust rules

- No new `unwrap()` or `expect()` outside tests and lock acquisition.
  Propagate the error or handle it.
- No formatting-only changes and no repository-wide reformatting.
- `cargo build --locked`. A build that resolves its own versions is not the
  build that was tested.

## Verification

```bash
cargo build --locked --all-targets
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --all --check
docker build -t nremote-server:dev .
```

The container build is the reference build. `libssl-dev` and `pkg-config` are
required for a host build; the image installs them itself.

## CI

Every workflow calls `NDDev-OpenNetwork/ci-workflows` pinned by full SHA and
runs on GitHub-hosted runners. That is not a fallback — a public repository must
never reach private self-hosted capacity, and public standard runners are
unmetered, so there is nothing to save by routing elsewhere.

## Governance

`.gds/repository.yaml` declares this repository's identity, portfolio, policy
profiles and required verification commands. It is the source; anything under
`.gds/` carrying a `GENERATED FILE` header is a projection of it and is never
edited by hand.
