#!/usr/bin/env bash
cp "$GITHUB_WORKSPACE/tools/support/release/jsr/consumer-deno.json" deno.json
deno run --config deno.json --allow-all consumer.ts "jsr:@vizejs/vize@${JSR_VERSION#v}"
