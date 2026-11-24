#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/check/validate-explicit-release-qualification.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ ! "$RELEASE_QUALIFICATION_REF" =~ ^refs/(heads/release/|tags/)v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  echo "Release qualification requires an immutable release/vSEMVER branch or vSEMVER tag." >&2
  exit 1
fi
