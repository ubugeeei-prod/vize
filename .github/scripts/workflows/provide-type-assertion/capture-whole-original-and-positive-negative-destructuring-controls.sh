#!/usr/bin/env bash
options=()
if [[ "$ROLE" == baseline ]]; then options+=(--ignored); fi
cargo test --locked --profile ci -p vize --test integration provide_type_assertion_cli::observe_complete_provide_assertion_packets -- "${options[@]}" --nocapture
