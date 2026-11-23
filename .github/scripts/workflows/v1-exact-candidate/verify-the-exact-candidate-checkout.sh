#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$CANDIDATE_SHA"
