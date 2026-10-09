# Retiring a failed unpublished immutable cut

References [#8346](https://github.com/ubugeeei-prod/vize/issues/8346) and
[#8335](https://github.com/ubugeeei-prod/vize/issues/8335).

The fixed source H is evidence. A source defect requires a new C/H/R and fresh qualification;
editing H or borrowing its successful jobs cannot repair the release. The official command can
retire a terminal failed/cancelled cut that never published, while retaining its original source,
pin and integration branches and every corresponding commit.

## Official command

Use the reviewed main-only Release Operator with operation `retire` and the original `source_pr`,
`source_head`, `source_tag`, `release_run` and `operator_run` identities. Its existing protected
environment credential must belong to the actual original and current triggering maintainer user.
The same public command is available locally:

```sh
vp run release --retire <source-PR> --head <H> --tag <tag> --run <R> --operator-run <original-entry>
```

The original publication R and operator entry must be terminal failed/cancelled. Allow the original
operator to return and release its own exact lease; an active or ambiguous lease is never stolen.
At least one completed failed/timed-out job in the original R or operator is required, with its
complete failure log. A wholly cancelled/skipped cut without that evidence is refused before
retirement acquires a lease. Publication phases are checked across every original attempt; every
phase must be cancelled/skipped with zero steps. Retries retain all earlier evidence and cannot
hide an earlier publication attempt.
The unchanged source must remain a maintainer-authored, unmerged draft with its exact C/H/body/pin.
The original integration must retain its source markers and version-only bytes, remain unmerged,
and have neither a queue entry nor auto-merge. Main may advance ordinary source changes while its
base version stays unchanged.

Any original tag or GitHub Release, including a draft Release, prohibits retirement. Every planned
exact npm/crates/editor version is derived from raw H and checked anonymously against the public
registries. Network/authentication/rate-limit errors, redirects, malformed responses and ambiguous
Marketplace fallbacks are refusals. Unknown provider payloads need a primary-response correction;
they never count as version absence. Marketplace reads the anonymous public Gallery query used by
[the official VS Code client](https://github.com/microsoft/vscode/blob/main/src/vs/platform/extensionManagement/common/extensionGalleryService.ts).
The fixed POST names one exact extension with filter 7 and `IncludeVersions` flag 1; no latest-only,
platform or publication-state filter is added. Retained method/query custody rejects flag 512 even
though that limited response has the same schema. Require one exact publisher/extension, total
result count 1, no continuation and complete version/platform rows; null paging tokens are normal.
Retain the whole response envelope and reject the requested version on every platform. This same
bounded anonymous query verifies that the exact published version exists after release. The
inventory is bounded to 2 MiB and 10,000
version/platform rows; missing-response bodies are bounded to 64 KiB. Truncation or partial inventory
is a refusal. Each anonymous read has one 30-second deadline covering the body.

The [primary response fixtures](../../tests/_fixtures/release/registry-provider-responses.json)
preserve the original anonymous response bytes and their digests: npm exact-version absence is a
JSON string `"version not found: VERSION"`; Open VSX also returns false `deprecated` and
`downloadable` metadata beside its exact missing-version error. Accept only those supported
shapes, retain their original JSON types, and reject foreign versions, truthy metadata or extra
fields. Parse and schema failures report the URL, HTTP status, original-byte digest/count and an
escaped body. This corrects the pre-lease refusal in
[operator 37986834551](https://github.com/ubugeeei-prod/vize/actions/runs/37986834551);
fresh Actions, protected merge and actual official retirement/publication remain required.

## Preservation and partial recovery

The command first creates an immutable `release-retired/<tag>` archive/reservation with an exact
absent lease. Archive parents retain the original pin/H/C and integration commit. The bounded
16 MiB tree preserves complete provider run/job/artifact metadata and relevant complete failure
logs. Artifact IDs, provider digests, sizes and expiry are retained; binary artifact bodies remain
in their original Actions runs with that expiry. There is no permanent artifact-byte custody or
replacement qualification claim. Pending old diagnostic rows are recorded honestly and can finish
independently.

Only an identical, complete archive can resume partial recovery. Under a newly acquired cooperative
lease, each PR is rechecked immediately before closure and all original identities are checked
again afterwards. Inventory pages and reservation loops check the same cooperative deadline;
nested target validation inherits its remaining allowance and original private absolute deadline.
A blocking external command can return after that deadline; its next checkpoint refuses further
mutation and allows normal lease cleanup. This is not a hard subprocess timeout. GitHub PR state
updates are separate from the Git archive transaction. A network
failure can leave an archive or one closed PR; that state blocks new preparation until the same
official command completes both original closures and every live guard. Original refs, titles,
bodies, source objects, main version and release tags are never rewritten or deleted.

## Preparing the replacement

An archive name alone reserves nothing. The official pinned minor command reauthenticates its
complete ledger, original closed source/integration, unchanged refs, original failed R/operator,
absent tag/Release/registry versions and absent lease. It skips each fully retired minor and chooses
the next unreserved version. Both the frozen source and moving-main integration receive that same
validated target before any metadata mutation. The internal target argument cannot choose an
arbitrary version; the Rust protocol independently verifies the authenticated next minor.

The original failed H `f58ba3592f378a6a40eea75b3d8040c6ad26ea0f` was prepared as `v0.438.0`
from `0.437.0` in source PR #8341 with R `37967359745` and entry `37966947051`. Once its official
retirement succeeds, the replacement minor would be `0.439.0`. These are historical source facts,
not a publication or future-candidate identity. Every new exact-H full gate, artifact check,
protected integration, tag/publication and installed public acceptance remains required.

Related: #8335, #6239 and #6830. Public acceptance for the original third-party reports remains
pending until a genuine replacement release succeeds.
