#!/usr/bin/env bash
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends time
# Instruction counts are optional: the driver notes their absence.
sudo apt-get install -y --no-install-recommends linux-tools-common "linux-tools-$(uname -r)" || true
sudo sysctl -w kernel.perf_event_paranoid=1 || true
