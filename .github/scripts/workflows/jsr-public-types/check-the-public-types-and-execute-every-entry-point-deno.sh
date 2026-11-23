#!/usr/bin/env bash
deno --version | tee runtime-version.txt
deno check --config deno.json consumer-types.ts 2>&1 | tee types.log
deno run --config deno.json --allow-all consumer.ts "jsr:@vizejs/vize@$PUBLIC_JSR_VERSION" --vite --core-only 2>&1 | tee runtime.log
