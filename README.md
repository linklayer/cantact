# CANtact
[![crates.io](https://img.shields.io/crates/v/cantact?label=cantact)](https://crates.io/crates/cantact)
[![crates.io](https://img.shields.io/crates/v/cantact-driver?label=cantact-driver)](https://crates.io/crates/cantact-driver)
[![PyPI](https://img.shields.io/pypi/v/cantact)](https://pypi.org/project/cantact/)
[![docs.rs](https://docs.rs/cantact-driver/badge.svg)](https://docs.rs/cantact-driver/)
![Rust Build](https://github.com/linklayer/cantact/workflows/Rust/badge.svg)
![Python Build](https://github.com/linklayer/cantact/workflows/Python/badge.svg)

Software support for CANtact devices. Includes a driver (see `driver/`), APIs, and a cross-platform command line interface.

## Getting a Device

CANtact Pro is currently a pre-launch project on CrowdSupply. You can subscribe on the [product page](https://www.crowdsupply.com/linklayer-labs/cantact-pro)
to get updates about the hardware release.

This tool should work fine with other CANtact/gs_usb compatible devices such as CANable.

## Installing

The CLI and driver are built using `cargo`, which can be installed using [rustup](https://rustup.rs/).

Once `cargo` is installed, use it to build and install the `can` binary:

```
git clone https://github.com/linklayer/cantact
cd cantact
cargo install --path .
```

### Setting udev Rules (Linux only)

On Linux, only root can access the device by default. This results in a `DeviceNotFound` error when trying to access the device as a normal user. 
To allow access for all users, create a file at `/etc/udev/rules.d/99-cantact.rules` which contains:
```
SUBSYSTEM=="usb", ATTRS{idVendor}=="1d50", ATTRS{idProduct}=="606f", MODE="0666"
```

Then reload the udev rules:
```
sudo udevadm control --reload-rules
sudo udevadm trigger
```


## Command Line Interface

The CLI is invoked using the `can` binary:

```
can help

can 0.1.0
Eric Evenchick <eric@evenchick.com>
Command line utilities for CANtact devices

USAGE:
    can [FLAGS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -v, --verbose    Print verbose debugging information
    -V, --version    Prints version information

SUBCOMMANDS:
    cfg     Set device configurations
    dump    Receive and display CAN frames
    help    Prints this message or the help of the given subcommand(s)
    send    Send a single CAN frame
```

The `can cfg` command is used to set the bitrate and other device settings. Once set, other commands will use these options.

For example, to set channels 0 and 1 to 500000 kbps, then dump all frames on all channels:

```
can cfg --channel 0 --bitrate 500000
can cfg --channel 1 --bitrate 500000
can dump
```

Use `can help [subcommand]` for additional documentation.

## Rust Support

The driver can be used from Rust by installing the [`cantact-driver` crate](https://crates.io/crates/cantact-driver).
Documentation for the crate can be found on [docs.rs](https://docs.rs/cantact-driver/).

## Python Support

CANtact supports Python 3.5+ on Windows, macOS, and Linux. The Python modules are hosted on [PyPI](https://pypi.org/project/cantact/).
Installation requires `pip 19.0+` (for manylinux2010 wheels).

Python end-users should not use this repository directly. Instead, install Python support using `pip`:

```
python3 -m pip -U pip
python3 -m pip install cantact
```

This will attempt to install a binary distribution for your system. If none exists, it will attempt to build
from source. This requires nightly rust, which can be enabled by running `rustup default nightly` before 
installing.

See the `examples/` folder for Python examples. [python-can](https://github.com/hardbyte/python-can/) supports
CANtact, and is recommended over using the `cantact` module directly. To install CANtact, `python-can`, 
and run a test:

```
python3 -m pip install cantact git+https://github.com/ericevenchick/python-can@cantact
can_logger.py -i cantact -c 0 -b 500000
```

### Building Python Support

Building Python support is only required if you want to make modifications to the `cantact` Python module, or if
you are using a platform that does not have packaged support.

Python support is implemented using [PyO3](https://github.com/PyO3/pyo3), and is gated by the `python` feature.
Thanks to [rust-setuptools](https://github.com/PyO3/setuptools-rust), the `cantact` Python module can be built
like any other Python module using `setuptools`. PyO3 requires nightly Rust, which can be configured using `rustup override`.

```
cd driver
rustup override set nightly
python setup.py build
```

Python builds for Windows, macOS, and manylinux are automated using [Github Actions](https://github.com/linklayer/cantact/actions?query=workflow%3APython).
Tagged releases are automatically pushed to PyPI.

## C / C++ Support

C / C++ support is provided by the driver. This is currently used to implement [BUSMASTER](https://rbei-etas.github.io/busmaster/) 
support on Windows.
