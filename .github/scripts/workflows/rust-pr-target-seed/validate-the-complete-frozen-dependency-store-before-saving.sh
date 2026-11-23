#!/usr/bin/env bash
vp install --frozen-lockfile --prefer-offline
git diff --exit-code -- package.json pnpm-lock.yaml pnpm-workspace.yaml
