#!/usr/bin/env bash
if [[ "$GITHUB_EVENT_NAME" == "merge_group" || "$GITHUB_EVENT_NAME" == "schedule" || "$GITHUB_EVENT_NAME" == "workflow_dispatch" ]]; then
  echo 'selected=true' >> "$GITHUB_OUTPUT"
else
  git diff --name-only "$BASE_SHA" "$GITHUB_SHA" > "$RUNNER_TEMP/vue1-text-document-paths.txt"
  if rg '^(crates/vize_glyph/src/native_doc(\.rs|/(expression(\.rs|/)|vue1_text(\.rs|/)|document\.rs|printer\.rs))|crates/vize_glyph/tests/(native_vue1_text_document|fixtures/native-vue1-text-doc-1\.0\.28\.json)|davinci/vize_l1/src/|davinci/vize_l0/src/source_frame\.rs|upstream/forks/oxc_parser/src/|tests/tooling/(native-vue1-text-document-(oracle|capture-workflow)\.test\.ts|support/vue1-(pinned-getter|text-document-judge)\.ts)|tests/_fixtures/reference/vue1/|\.github/actions/test-native-(vue1|historical)-text-document|\.github/workflows/pr-source-checks\.yml)' "$RUNNER_TEMP/vue1-text-document-paths.txt"; then
    echo 'selected=true' >> "$GITHUB_OUTPUT"
  else
    echo 'selected=false' >> "$GITHUB_OUTPUT"
  fi
fi
