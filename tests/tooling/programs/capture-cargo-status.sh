#!/usr/bin/env bash
cargo() {
  printf '%s\n' '{"reason":"compiler-artifact","filenames":[]}'
  return "$MOCK_CARGO_STATUS"
}

source "$1"
