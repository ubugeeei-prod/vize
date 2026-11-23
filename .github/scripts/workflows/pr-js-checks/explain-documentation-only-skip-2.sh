#!/usr/bin/env bash
echo 'Oxlint project qualification skipped because this PR changes no package, compiler, or unknown source paths.' >> "$GITHUB_STEP_SUMMARY"
