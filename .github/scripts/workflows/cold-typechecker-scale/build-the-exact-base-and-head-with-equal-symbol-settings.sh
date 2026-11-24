#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/cold-typechecker-scale/build-the-exact-base-and-head-with-equal-symbol-settings.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
git fetch --no-tags --depth=1 "https://github.com/$GITHUB_REPOSITORY.git" "$BASE_SHA"
git worktree add --detach "$RUNNER_TEMP/base" "$BASE_SHA"
mkdir -p "$RUNNER_TEMP/cold-scale"
git rev-parse HEAD > "$RUNNER_TEMP/cold-scale/head.sha"
git -C "$RUNNER_TEMP/base" rev-parse HEAD > "$RUNNER_TEMP/cold-scale/base.sha"
for source in base head; do
  source_root="$GITHUB_WORKSPACE"
  if [[ "$source" == base ]]; then source_root="$RUNNER_TEMP/base"; fi
  node tools/benchmarks/scripts/cold-typechecker-scale-build.ts --source-root "$source_root" --target-dir "$RUNNER_TEMP/cold-builds/normal-$source" --binary "$RUNNER_TEMP/cold-scale/binaries/normal-$source/vize" --receipt "$RUNNER_TEMP/cold-scale/builds/normal-$source.json" --role "normal-$source"
done
node tools/benchmarks/scripts/cold-typechecker-scale-build.ts --base-receipt "$RUNNER_TEMP/cold-scale/builds/normal-base.json" --head-receipt "$RUNNER_TEMP/cold-scale/builds/normal-head.json" --receipt "$RUNNER_TEMP/cold-scale/builds/normal-pair.json"
