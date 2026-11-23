#!/usr/bin/env bash
set -euo pipefail
mkdir -p target/vize-tests/async-component
git rev-parse HEAD > target/vize-tests/async-component/producer.sha
node --version > target/vize-tests/async-component/node.version
