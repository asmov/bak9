# Building bak9

These steps are designed for cross-compilation from an Ubuntu 24 linux distro,
using a Windows 11 Pro VM and hardware running macOS Sonoma on an M3 CPU.

## Rust Targets

To install a target:
```bash
rustup target add $TARGET
rustup toolchain install stable-$TARGET 
```

- x86_64-unknown-linux-gnu
- armv7-unknown-linux-gnueabihf
- aarch64-unknown-linux-gnu
- aarch64-apple-darwin
- x86_64-pc-windows-msvc

## Requirements

Debian packaging requires:
```bash
sudo apt install \
pkg-config \
build-essential \
crossbuild-essential-arm64 \
crossbuild-essential-armhf
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
cargo install cargo-deb
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


## Windows

### VM Setup
1. Install a Windows 11 VM. Update it.
2. Install OpenSSH Server feature:
    - Reference: https://learn.microsoft.com/en-us/windows-server/administration/openssh/openssh_install_firstuse
    1. Settings > System > Features > View Features > Add Feature > "OpenSSH Server"
    2. Services > "OpenSSH Server" > Properties > Start: Automatically
    3. Configure Powershell as the default OpenSSH shell: `New-ItemProperty -Path "HKLM:\SOFTWARE\OpenSSH" -Name DefaultShell -Value "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe" -PropertyType String -Force`
    4. Configure OpenSSH for pubkey authentication: [Microsoft: Key-based authentication in OpenSSH for Windows](https://learn.microsoft.com/en-us/windows-server/administration/openssh/openssh_keymanagement)


### Development Setup
5. Install [rustup](https://rustup.rs).
    - After installing VS 2022, manually install the *C++ Desktop Development* component to it. 
6. Install [WiX v3](https://github.com/wixtoolset/wix3/releases) 
7. Install cargo-wx: `cargo install cargo-wix`


### Building & Packaging
```
cargo build --release
cargo wix
```