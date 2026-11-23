#!/usr/bin/env bash
baseline_sha=9157b9fa83dbdc98100be084be288f21d0d1bc18
baseline_root="$RUNNER_TEMP/nuxt-kit-before-source"
git fetch --no-tags --depth=1 origin "$baseline_sha"
git worktree add --detach "$baseline_root" "$baseline_sha"
(cd "$baseline_root" && pnpm install --frozen-lockfile --ignore-scripts)
vp node tools/support/security/nuxt-kit-census.ts "$RUNNER_TEMP/nuxt-public-kit-proof/before" "$baseline_root"
