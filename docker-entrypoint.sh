#!/bin/sh
# hbbs creates id_ed25519, id_ed25519.pub and the SQLite database in the
# working directory on first start, with whatever the process umask allows.
# The default 022 makes the private key world-readable, and a private key at
# 0644 on a host that also runs other containers is a finding waiting to be
# written up. Nothing here needs group or other access to anything it creates,
# so the umask says so once, at the only place every command passes through.
umask 0077
exec "$@"
