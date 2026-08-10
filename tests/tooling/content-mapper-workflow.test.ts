import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

const UPSTREAM_SHA = "bddd2162710e50281fa838456a875fd59ee7c91f";

test("Content Mapper conformance pins and runs the exact upstream project path", () => {
  const workflow = readRepoFile(".github", "workflows", "content-mapper-conformance.yml");
  const job = workflowJobBody(workflow, "exact-tsgo-project");

  assert.match(workflow, /pull_request:\n\s+branches: \[main\]\n\s+paths:/);
  for (const relevantPath of [
    '".github/workflows/content-mapper-conformance.yml"',
    '"crates/vize/src/commands/content_mapper.rs"',
    '"crates/vize_canon/src/batch.rs"',
    '"crates/vize_canon/src/batch/virtual_project.rs"',
    '"crates/vize_canon/src/batch/virtual_project/content_mapper*"',
    '"crates/vize_canon/src/lib.rs"',
    '"crates/vize_canon/src/lsp_client.rs"',
    '"crates/vize_canon/src/lsp_client/**"',
    '"crates/vize_canon/tests/lsp_import_resolution.rs"',
    '"crates/vize_canon/src/virtual_ts/**"',
    '"crates/vize_maestro/src/ide/**"',
    '"crates/vize/tests/content_mapper_tsgo_cli.rs"',
    '"crates/vize/tests/content_mapper_tsgo_build.rs"',
    '"crates/vize/tests/fixtures/content_mapper_project/**"',
    '"npm/cli/bin/vize"',
    '"npm/cli/package.json"',
    '"npm/cli/src/**"',
    '"npm/native/**"',
    '"tools/npm/prepare-publish-manifest.mjs"',
    '"tools/npm/smoke-release-install.mjs"',
    '"tools/npm/smoke-release-runtime.mjs"',
  ]) {
    assert.match(workflow, new RegExp(relevantPath.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
  }

  assert.match(workflow, new RegExp(`CONTENT_MAPPER_TSGO_SHA: "${UPSTREAM_SHA}"`));
  assert.match(job, /repository: microsoft\/typescript-go/);
  assert.match(job, /ref: \$\{\{ env\.CONTENT_MAPPER_TSGO_SHA \}\}/);
  assert.match(job, /uses: actions\/setup-go@[0-9a-f]{40}\s+# v6\.1\.0/);
  assert.match(
    job,
    /uses: \.\/\.github\/actions\/setup-rust-sticky-cache\n\s+with:\n\s+key: content-mapper-conformance\n\s+cache-key-suffix: \$\{\{ runner\.os \}\}-\$\{\{ runner\.arch \}\}/,
  );
  assert.match(job, /go-version-file: typescript-go-content-mapper\/go\.mod/);
  assert.match(job, /go build -tags=noembed -trimpath -o "\$RUNNER_TEMP\/tsgo" \.\/cmd\/tsgo/);
  assert.match(job, /cp internal\/bundled\/libs\/\*\.d\.ts "\$RUNNER_TEMP\/"/);
  assert.match(job, /VIZE_TEST_CONTENT_MAPPER_TSGO: \$\{\{ runner\.temp \}\}\/tsgo/);
  assert.match(
    job,
    /VIZE_TEST_CONTENT_MAPPER_JAVASCRIPT_TSC: \$\{\{ github\.workspace \}\}\/npm\/cli\/node_modules\/\.bin\/tsc/,
  );
  assert.match(job, /cargo test -p vize --test content_mapper_tsgo_cli -- --nocapture/);
  assert.match(job, /cargo test -p vize --test content_mapper_tsgo_build -- --nocapture/);
  assert.match(job, /vp run --filter '\.\/npm\/native' build:ci/);
  assert.match(job, /\(cd npm\/cli && vp pack\)/);
  assert.match(job, /vp exec napi create-npm-dirs/);
  assert.match(job, /cp "\$binary" "npm\/\$target\/"/);
  assert.match(
    job,
    /VIZE_TEST_CONTENT_MAPPER_TSGO: \$\{\{ runner\.temp \}\}\/tsgo[\s\S]*smoke-release-install\.mjs --prepare-manifests --content-mapper-checks[\s\S]*npm\/native npm\/native\/npm\/\*[\s\S]*npm\/cli/,
  );
  assert.match(job, /TSGO_PATH: \$\{\{ runner\.temp \}\}\/tsgo/);
  assert.match(job, /cargo test -p vize_canon --test lsp_import_resolution -- --nocapture/);
  assert.match(job, /cargo test -p vize_maestro/);
});
