#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$EXPECTED_SHA"
