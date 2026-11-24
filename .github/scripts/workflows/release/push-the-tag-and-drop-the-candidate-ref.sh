#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/push-the-tag-and-drop-the-candidate-ref.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
git push origin "HEAD:refs/tags/$RELEASE_TAG"
git push origin ":$RELEASE_CANDIDATE"
