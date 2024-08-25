#!/bin/bash
set -ex

yum install libusb-devel libusbx-devel

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
export PATH="$HOME/.cargo/bin:$PATH"

cd driver/

for PYBIN in /opt/python/{cp38-cp38,cp39-cp39,cp310-cp310,cp311-cp311,cp312-cp312}/bin; do
    export PYTHON_SYS_EXECUTABLE="$PYBIN/python"

    "${PYBIN}/python" -m pip install -U setuptools wheel setuptools-rust
    "${PYBIN}/python" setup.py bdist_wheel
done

for whl in dist/*.whl; do
    auditwheel repair "$whl" -w dist/
done

# remove non-manylinux wheels
rm dist/*-linux*.whl
