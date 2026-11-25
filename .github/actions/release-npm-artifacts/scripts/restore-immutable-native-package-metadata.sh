#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/release-npm-artifacts/scripts/restore-immutable-native-package-metadata.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
git fetch --no-tags --depth=1 origin "refs/tags/$RELEASE_TAG"
for file in pnpm-workspace.yaml npm/native/package.json npm/fresco-native/package.json; do
  git show "FETCH_HEAD:$file" > "$file"
done
