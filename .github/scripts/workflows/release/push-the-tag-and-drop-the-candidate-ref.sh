#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/push-the-tag-and-drop-the-candidate-ref.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
# Tag only the candidate commit this run stamped and every artifact was built from.
head=$(git rev-parse HEAD)
if [[ ! "$RELEASE_SHA" =~ ^[0-9a-f]{40}$ || "$head" != "$RELEASE_SHA" ]]; then
  echo "Refusing to tag $RELEASE_TAG: the checkout is at '$head', the release candidate is '$RELEASE_SHA'" >&2
  exit 1
fi
git push origin "HEAD:refs/tags/$RELEASE_TAG"
git push origin ":$RELEASE_CANDIDATE"
