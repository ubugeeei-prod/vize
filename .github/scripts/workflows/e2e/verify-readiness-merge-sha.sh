#!/usr/bin/env bash
test "$(git rev-parse HEAD)" = "$EXPECTED_SHA"
echo "Tested merge SHA $EXPECTED_SHA for source head $SOURCE_HEAD_SHA" >> "$GITHUB_STEP_SUMMARY"
