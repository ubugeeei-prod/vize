#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/viteplus-task-cache/check-real-installed-tasks-and-cache-invalidation.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [ -n "$VIZE_TASK_CACHE_PUBLIC_VERSION" ]; then
  "$VIZE_TASK_CACHE_RUNTIME" npm/builder/vite/scripts/check-task-cache.ts "$VIZE_TASK_CACHE_PUBLIC_VERSION"
else
  "$VIZE_TASK_CACHE_RUNTIME" npm/builder/vite/scripts/check-task-cache.ts
fi
