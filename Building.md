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


## Building for Windows

1. Install a Windows 11 VM.
2. Update system.
2. Install the OpenSSH Server feature:
    - Reference: https://learn.microsoft.com/en-us/windows-server/administration/openssh/openssh_install_firstuse
    1. Settings > System > Features > View Features > Add Feature > "OpenSSH Server"
    2. Services > "OpenSSH Server" > Properties > Start: Automatically
3. To configure Powershell as the default shell:
    1. SSH into Windows
    2. Run: `powershell`
    3. Run: `New-ItemProperty -Path "HKLM:\SOFTWARE\OpenSSH" -Name DefaultShell -Value "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe" -PropertyType String -Force`
4. Install:
    - rust (rustup)