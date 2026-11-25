#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-lsp-vs-volar/scripts/build-the-compare-ref.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
git fetch --no-tags --depth 1 "https://github.com/${GITHUB_REPOSITORY}.git" "$COMPARE_REF"
git worktree add "$RUNNER_TEMP/compare" FETCH_HEAD
(cd "$RUNNER_TEMP/compare" && CARGO_TARGET_DIR="$GITHUB_WORKSPACE/target" cargo build --profile ci-opt -p vize)
cp target/ci-opt/vize "$RUNNER_TEMP/vize-compare"
(cd "$RUNNER_TEMP/compare" && CARGO_TARGET_DIR="$GITHUB_WORKSPACE/target" cargo build --profile ci-opt -p vize_vitrine --features napi,legacy)
cp target/ci-opt/libvize_vitrine.so "$RUNNER_TEMP/vize-vitrine-compare.node"
