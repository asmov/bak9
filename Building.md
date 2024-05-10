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