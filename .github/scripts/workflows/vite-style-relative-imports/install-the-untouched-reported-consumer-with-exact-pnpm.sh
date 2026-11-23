#!/usr/bin/env bash
npx --yes pnpm@12.9.0 install --frozen-lockfile
ln -s "$GITHUB_WORKSPACE/tests/_fixtures/_projects/vize-style-relative-imports-host/node_modules/lightningcss" node_modules/lightningcss
