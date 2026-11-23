#!/usr/bin/env bash
cargo run --locked --profile ci -p vize_l4 --example vue2_ir_template > "$VIZE_VUE2_IR_CAPTURE" 2> target/vize-tests/vue2-ir/capture.stderr
