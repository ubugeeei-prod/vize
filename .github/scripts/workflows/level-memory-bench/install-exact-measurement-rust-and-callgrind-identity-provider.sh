#!/usr/bin/env bash
rustup toolchain install 1.100.0 --profile minimal
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends valgrind
