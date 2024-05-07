# Building bak9

Most of these steps are for cross-compilation.


## Requirements

*Tested against Ubuntu Desktop 23*

Debian packaging requires:
```bash
sudo apt install \
pkg-config build-essential \
cross-build-essential-arm64 \
cross-build-essential-armhf
```

Snap packaging requires:
```bash
sudo snap install snapcraft --classic
```

Cross-compiling requires:
```bash
cargo install cross
```

Debian packaging requires:
```bash
cargo install carg-deb
```

RPM packaging requires:
```bash
cargo install cargo-generate-rpm
```

## Cross-Compiling

```bash
# for each rustup TARGET ...
cross --release --target=$TARGET
```

## Packaging for Debian

Requires the `cross-build-essential-` packages

```bash
# for each linux rust TARGET, after compiling ...
cargo deb --target=$TARGET --no-build
```


## Packaging for Snap

From the project root:
```bash
snapcraft
```


## Packaging for RPM

```bash
# for each linux rust TARGET, after compiling ...
cargo generate-rpm --target=$TARGET
```