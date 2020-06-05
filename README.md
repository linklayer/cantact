# CANtact

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

## Python Support

Python end-users should not use this repository directly. Instead, install Python support using pip:

```
python3 -m pip install cantact
```

See the `examples/` folder for Python examples. [python-can](https://github.com/hardbyte/python-can/) supports
CANtact, and is recommended over using the `cantact` module directly.

```
python3 -m pip install python-can cantact
can_logger.py -i cantact -c 0 -b 500000
```

### Building Python Support

Building Python support is only required if you want to make modifications to the `cantact` Python module.

Python support is implemented using [PyO3](https://github.com/PyO3/pyo3), and is gated by the `python` feature.
Thanks to [rust-setuptools](https://github.com/PyO3/setuptools-rust), the `cantact` Python module can be built
like any other Python module using `setuptools`. PyO3 requires nightly Rust, which can be configured using `rustup override`.

```
cd driver
rustup override set nightly
python setup.py build
```

To build [manylinux](https://github.com/pypa/manylinux) compliant wheels, use `manylinux-build.sh`.
This uses the `manylinux2010` Docker image to build the wheels into `dist/`.