#!/usr/bin/env bash
cargo test --locked -p vize_l1 --lib container::vue
cargo test --locked -p vize_atelier_sfc --lib parse::tests
