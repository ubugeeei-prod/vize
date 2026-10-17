# Published Vue script suppression acceptance

Issue [#9291](https://github.com/ubugeeei-prod/vize/issues/9291) stays open until the
next minor release passes this consumer. Source merge #9293 and its four real
Linter tests establish source correctness; they do not establish published npm
behavior.

Run the same frozen packet against an exact public minor version:

```sh
node tools/support/release/vue_script_suppressions/main.mjs 0.457.0 /tmp/vize-public-suppressions-0.457.0
```

The output path must be new and outside the checkout. Installation uses only
`vize` and `@vizejs/native` at that exact version from the public npm registry.
The shared public acceptance verifier authenticates their archives and the
actually loaded platform native package against npm integrity and installed
files. Runtime overrides, workspace bindings, linked packages, and reused output
directories are refused. No checkout binary, provider override, or Cargo build
is used.

The immutable fixture expands the four source regression tests into 72 CLI
inputs: 12 positive, clean, and unrelated-rule controls; 48 pragma forms and
prefixes; eight cross-name enable controls; and four controls that preserve all
three other enabled findings. The derived law checks every finding's identity,
severity, counts, process result, and unchanged inputs. It grants no independent
message or coordinate oracle, full n8n configuration qualification, template
parity, or release-wide acceptance.

Every case runs before aggregate rejection. Exclusive process packets preserve
captured stdout and stderr as base64, status, signal, errors, original and observed
input hashes, argv, and installed binary identity. Separate observation files
record the derived law without rewriting raw process evidence. CLI captures are
capped at 8 MiB and the version probe at 1 MiB per stream. Capture limits and
`rawCaptureComplete` are recorded; an error such as `ENOBUFS` marks capture
incomplete and rejects the run. A truncated capture cannot establish a whole
response. Package archives,
metadata, lockfile, version probe, helper hashes, and the final receipt are kept
beside them. A strict RED run is evidence of failure, not a permitted baseline.
Only the version and fresh output directory change for the next minor replay.

The actual public `0.457.0` baseline on 2026-10-17 captured all 72 processes:
16 contracts passed and 56 failed, with no signals or spawn errors. The failures
were all 48 alias pragma variants, four native-disable/Vue-enable controls, and
four other-rule preservation controls. The installed darwin-arm64 native SHA256
was `2d993129626719ca251eb1f7db804e27cfc37ba4e2c5bb418c1f8a64fb4f52c4`.
The frozen fixture SHA256 is
`56e707cee8d2ced783a1ff7c6d87e79e12cd5f8d754a1813728b3f9d3b492be5`;
the strict RED receipt SHA256 is
`8baa964e13ae8568b18f1fed44d6af881f36f57e0951e321c93d5ed94e8b5d81`.
Complete evidence is retained at
`/private/tmp/vize-public-suppressions-0.457.0-9291`, with a per-case custody
manifest at `/tmp/vize-9291-public-baseline-review.json` (SHA256
`9acc04fa7f71b943d1cbb63c467f4953831f66be627154c37248b23e47d8f311`).
These receipts establish the old published failure only; a future version still
requires its own exact replay and official release verification.
The baseline predates the explicit capture-limit metadata: none of its processes
had capture errors. Its exact executed helper bytes are retained in its `helpers`
directory. The subsequent metadata-only addition changes no fixture or verdict.

The small unit suite tests custody and rejection laws without claiming native
execution:

```sh
node --test tests/tooling/released-vue-script-suppressions.test.mjs
```
