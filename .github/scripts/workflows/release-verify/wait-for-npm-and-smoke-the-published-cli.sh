#!/usr/bin/env bash
read -ra packages <<< "$RELEASE_PACKAGES"
node tools/support/release/verify_published/main.ts "$RELEASE_VERSION" "${packages[@]}"
