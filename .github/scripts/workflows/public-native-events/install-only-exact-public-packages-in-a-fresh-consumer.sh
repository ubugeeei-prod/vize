#!/usr/bin/env bash
mkdir public-event-consumer
cd public-event-consumer
npm init -y
npm install --ignore-scripts --no-audit --no-fund --save-exact "@vizejs/native@$PUBLIC_VERSION" @vue/compiler-dom@3.5.44 vue@3.5.44
cp package-lock.json "$RUNNER_TEMP/event-receipts/package-lock.json"
npm view "@vizejs/native@$PUBLIC_VERSION" --json > "$RUNNER_TEMP/event-receipts/npm-native-metadata.json"
