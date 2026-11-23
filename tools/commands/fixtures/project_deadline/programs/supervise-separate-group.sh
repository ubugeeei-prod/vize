#!/usr/bin/env bash
set -m
node "$@" &
wait
