# Isolate mock npm registry reads from publisher state

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The full tooling run failed while parsing the mock publisher's JSON state in
`publish-npm-package.test.ts`. Exact-version and dist-tag visibility requests
now run concurrently. Both fake curl processes read, incremented counters in,
and rewrote the publisher's same file. Their writes could overwrite each other's
updates or leave malformed JSON. This also affected the serial directory
publisher: its per-package visibility requests still run concurrently.

The shared fake registry now reads publisher state without modifying it and
records requests in an append-only URL log. The staged-publish control derives
its original three version checks and one tag check from that log and requires
the entire publisher state to remain exactly `{ publishCalls: 1 }`. Its tag is
available while the exact version finishes propagating, independent of which
request process runs first. Obsolete `vp pm view` mocks were removed from that
control.

The directory-selection control uses one atomically published record per
package. Registry requests only read those records; they reject an unpublished
package or a different requested version. The test still requires the exact
published package set, serial package order, and package versions. The existing
directory without a manifest remains excluded.

No production publishing code, concurrency limit, retry limit, provenance
handling, or exact-version-plus-tag acceptance condition changes. The release
continues to use its existing bounded concurrency. Existing negative controls
still reject hidden staged versions and incorrect tags and retain child
failures and diagnostics.

Local validation:

- `node --test tests/tooling/publish-npm-package*.test.ts`: 30 passed; the
  existing opt-in benchmark was skipped. This includes actual concurrent
  package publishers, provenance, retry, tag propagation and rerun controls.
- A bounded 32-request probe returned all 16 exact-version and 16 tag responses,
  retained every request once, and left publisher state byte-identical.
- Narrow formatting, lint and diff checks passed. The staged test also passed
  after strengthening its complete-state assertion.

These observations qualify the local mock repair. They do not claim a complete
hosted tooling result or a release publication; the combined Actions run remains
the integration check.
