#!/bin/bash
printf "%s\n" "$*" >> "$RUNNER_LOG"
printf "rust-script @@VERSION@@\n"
