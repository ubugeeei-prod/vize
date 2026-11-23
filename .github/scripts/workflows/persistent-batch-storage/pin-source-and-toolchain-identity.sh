#!/usr/bin/env bash
mkdir -p target/persistent-storage-evidence
git rev-parse HEAD > target/persistent-storage-evidence/source.sha
rustc --version --verbose > target/persistent-storage-evidence/rustc.txt
cp Cargo.lock target/persistent-storage-evidence/
