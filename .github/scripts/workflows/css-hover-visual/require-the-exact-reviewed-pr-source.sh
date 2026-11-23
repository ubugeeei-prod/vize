#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
