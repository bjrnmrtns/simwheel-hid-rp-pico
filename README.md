## Table of Contents
1. [Requirements](#requirements)
2. [Setup](#setup)
    1. [System Setup](#system-setup)

## Requirements
* Raspberry Pi Pico
* Rust Toolchain ([`cargo`][8], [`rustup`][15])

## Setup
### System Setup
1. Install Rust
2. Install Cortex-M Target Toolchain Support for rust
```shell
# Install `thumbv6m-none-eabi` Target for `rp2040`
$ rustup target add thumbv6m-none-eabi
```

3. Install additional dependencies
```shell
# Install Linux Dependencies
$ sudo apt install -y libusb-1.0-0-dev libudev-dev

# Install `elf2uf2`
$ cargo install elf2uf2

# Install udevil
$ sudo apt-get install udevil
```

4. Install [`flip-link`][6]
```shell
# Install `flip-link`
$ cargo install flip-link
```

## Usage
1. Find the correct device
```
udevil mount /dev/sdX1
```

## Testing
1. Install jstest
```
sudo apt-get install joystick
```
2. Run jstest
```
jstest --normal /dev/input/js0
```

