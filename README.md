## Environment Setup

### Rust Toolchain
Switch to Rust nightly and add `thumbv7em-none-eabi`
```
rustup default nightly
rustup target add thumbv7em-none-eabi
```
At the time of writing, the nightly version is 2022-04-08

### QEMU
Install the patched QEMU that supports STM32F407-Discovery from https://xpack.github.io/qemu-arm/. The fastest way would be running the following commands.

```
sudo apt install npm
sudo npm install --global xpm@latest
xpm install --global @xpack-dev-tools/qemu-arm@latest --verbose
```

One can find the patched QEMU at
```
~/.local/xPacks/@xpack-dev-tools/qemu-arm/6.2.0-2.1/.content/bin/qemu-system-gnuarmeclipse
```

Add it to the `PATH`, or link it to `/usr/bin` as follows
```
sudo ln -s \
    ~/.local/xPacks/@xpack-dev-tools/qemu-arm/6.2.0-2.1/.content/bin/qemu-system-gnuarmeclipse \
    /usr/bin/qemu-system-gnuarmeclipse
```

## Build and Run

- To build the image in debug mode, execute `make debug`. The built image is `image-debug.elf`.
- To run the debug image, execute `make run-debug`. At the time of writing, the debug build *won't* print correctly.
- To build the image in release mode, execute `make release`. The built image is `image-release.elf`.
- To run the release image, execute `make run-release`. At the time of writing, the release build print correctly.
- To get the disassembly, execute `make dump-debug` or `make dump-release`. The dumped disassembly is in `dump-debug.asm` or `dump-release.asm`. (Requires `arm-none-eabi-*` toolchain.)
- To clean up, run `make clean`.
