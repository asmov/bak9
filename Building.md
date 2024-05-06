# Building bak9

Most of these steps are for cross-compilation.


## Requirements

Cross-compiling requires:
```bash
cargo install cross
```

Debian packaging requires:
```bash
cargo install carg-deb
```

Debian packaging requires:
```bash
sudo apt install \
pkg-config build-essential \
cross-build-essential-arm64 \
cross-build-essential-armhf
```

## Compiling

```bash
# for each rustup TARGET ...
cross --release --target=$TARGET
```

# Packaging for Debian

Requires the `cross-build-essential-` packages

```bash
# for each Debian rustup TARGET, after compiling ...
cargo deb --target=$TARGET --no-build
```

