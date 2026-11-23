#!/usr/bin/env bash
if ! command -v strace >/dev/null 2>&1; then
  sudo apt-get update
  sudo apt-get install -y strace
fi
