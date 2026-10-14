#!/bin/sh
# Downloads the prebuilt cargo-deny and cargo-machete releases named by CARGO_DENY_VERSION
# and CARGO_MACHETE_VERSION into .ci-bin/, verifying each archive against its published
# SHA-256. POSIX sh on purpose: it runs in .woodpecker/deps.yaml's alpine step (busybox).
set -eu

case "$(uname -m)" in
  x86_64) deny_target=x86_64-unknown-linux-musl; machete_target=x86_64-unknown-linux-musl ;;
  # cargo-machete ships aarch64 Linux only as a glibc build; it runs in the rust image.
  aarch64) deny_target=aarch64-unknown-linux-musl; machete_target=aarch64-unknown-linux-gnu ;;
  *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

# fetch <base-url> <archive-stem>: download, verify, and unpack <archive-stem>/<tool>.
fetch() {
  wget -q "$1/$2.tar.gz" "$1/$2.tar.gz.sha256"
  echo "$(cat "$2.tar.gz.sha256")  $2.tar.gz" | sha256sum -c -
  tar -xzf "$2.tar.gz"
}

mkdir -p .ci-bin
cd .ci-bin

stem="cargo-deny-$CARGO_DENY_VERSION-$deny_target"
fetch "https://github.com/EmbarkStudios/cargo-deny/releases/download/$CARGO_DENY_VERSION" "$stem"
mv "$stem/cargo-deny" .

stem="cargo-machete-v$CARGO_MACHETE_VERSION-$machete_target"
fetch "https://github.com/bnjbvr/cargo-machete/releases/download/v$CARGO_MACHETE_VERSION" "$stem"
mv "$stem/cargo-machete" .
