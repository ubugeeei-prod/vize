# `@vizejs/marquette`

Typed application marquettes for every Vize target.

> Experimental: serialized contracts are versioned, but the TypeScript API may
> evolve while target adapters are being validated.

Marquette is the small, language-neutral description shared by web, native,
desktop, terminal, backend, transport, testing, and gallery tooling. This
package provides literal-preserving TypeScript authoring. Runtime validation is
available through a separate entry, while canonicalization and compatibility
analysis remain focused layers.

## Author a marquette

```ts
import { defineApplicationMarquette } from "@vizejs/marquette";

export default defineApplicationMarquette({
  application: "shop",
  targets: ["web", "native"],
  environments: [
    {
      id: "web",
      target: "web",
      consumer: "client",
      runtime: "browser",
    },
    {
      id: "native",
      target: "native",
      consumer: "client",
      runtime: "native",
    },
    {
      id: "server",
      target: "web",
      consumer: "server",
      runtime: "rust",
    },
  ],
  backends: [
    {
      id: "api",
      family: "rust",
      environment: "server",
    },
  ],
  protocols: [
    {
      id: "api.query",
      family: "schema-query",
      backend: "api",
    },
  ],
  routes: [
    {
      id: "home",
      path: "/",
      environment: "web",
      rendering: "hybrid",
      backend: "api",
      protocol: "api.query",
    },
  ],
});
```

Environment dependencies, backend owners, protocol owners, and route
references are checked against identifiers declared in the same object. A
misspelled reference is therefore an authoring error rather than a runtime
surprise.

## Runtime validation

Use the dedicated validation entry when contracts cross file, process, or
language boundaries:

```ts
import { validateApplicationMarquette } from "@vizejs/marquette/validate";

const diagnostics = validateApplicationMarquette(shop);
for (const diagnostic of diagnostics) {
  console.error(diagnostic.code, diagnostic.path, diagnostic.message);
}
```

Validation is deterministic, does not mutate the authored object, and reports
stable codes shared with the native contract implementation. Keeping it in a
separate entry means authoring-only applications do not include validation
code.

## Adapter capability negotiation

Adapters publish inclusive capability-version ranges independently from the
application marquette. Application capability definitions are the authority
for requirements; an undeclared requirement fails closed even when an adapter
offers a capability with the same identifier. Adapter-only capabilities are
allowed and remain additive until an application requires them.

```ts
import {
  negotiateAdapterCapabilities,
  type AdapterCapabilityManifest,
} from "@vizejs/marquette/adapter";

const adapter = {
  adapter: "terminal.fresco",
  capabilities: [{ id: "terminal.color", minVersion: 1, maxVersion: 3 }],
} satisfies AdapterCapabilityManifest;

const result = negotiateAdapterCapabilities(
  marquette,
  marquette.environments?.[0]?.capabilities ?? [],
  adapter,
);
if (!result.compatible) {
  console.error(result.diagnostics, result.mismatches);
}
```

Exact and inclusive-bound matches are compatible. Missing support,
requirements below an adapter's minimum, and requirements above its maximum
use the stable `missing-capability`, `version-below-minimum`, and
`version-above-maximum` codes. Adding a support range or widening either bound
is additive; removing support or narrowing either bound is breaking. Both
negotiation and compatibility reports are sorted deterministically and do not
mutate their inputs. Compatibility reports expose validation diagnostics for
both manifests and omit changes when either input is invalid. The published manifest schema is available from
`@vizejs/marquette/adapter/schema`.

## Test-run evidence

Release-bound test-run evidence records live in their own lazy entries so
deployment tooling can bind a `tests` check to retained, immutable facts:

```ts
import { defineTestRunEvidence } from "@vizejs/marquette/test-run";
import { validateTestRunEvidence } from "@vizejs/marquette/test-run/validate";
import { testRunAdmissionId } from "@vizejs/marquette/test-run/canonical";
import { admitTestRun, decideTestRunAdmission } from "@vizejs/marquette/test-run/admission";
import { verifyTestRunCheck } from "@vizejs/marquette/test-run/check";
import { verifyTestRunTransition } from "@vizejs/marquette/test-run/transition";

const diagnostics = validateTestRunEvidence(evidence);
const admissionId = await testRunAdmissionId(evidence); // test-run:<sha256>
const rejections = await admitTestRun(evidence, candidate, admissionId, now);
const decision = await decideTestRunAdmission(evidence, candidate, admissionId, now);
if (!decision.allowed) {
  console.error(decision.denialCodes); // e.g. ["record-expired"]
}
const release = await verifyTestRunCheck(retainedCheck, candidate, evidence, now);
const journal = await verifyTestRunTransition(nextTransition, chainTip);
```

Validation shares its `VIZE_MARQUETTE_1xx` codes, paths, and ordering with the
native implementation, and the canonical entry produces byte-identical
serialization and SHA-256 fingerprints, proven by the shared fixtures in
`tests/fixtures/test-run-evidence/`. Admission decisions carry the stable,
append-only denial-code vocabulary shared by every backend family; the same
fixtures pin every decision so a JavaScript, Rust, Go, or JVM gate denies for
identical machine-readable causes. The check entry replaces every generic
test-result reference in release evidence: a retained `vize.test-run.check`
record may only name the exact `test-run:<sha256>` admission id, bind the six
candidate facts, and record an observer independent from the runner. The
transition entry makes each release decision and the complete accepted
anti-replay state one durable atomic record, chained by canonical SHA-256
fingerprints; hosts persist it with a write-then-atomic-rename and verify the
recovered tip before deciding anything new. The published record schema is
available from `@vizejs/marquette/test-run/schema`, the decision contract from
`@vizejs/marquette/test-run/admission/schema`, the tests-check contract from
`@vizejs/marquette/test-run/check/schema`, and the transition contract from
`@vizejs/marquette/test-run/transition/schema`.

## Guarantees

- Exact identifiers remain available as `EnvironmentId`, `BackendId`,
  `ProtocolId`, and `RouteId` unions.
- `defineApplicationMarquette` returns its input without allocation or
  normalization.
- The published application-contract schema is available from
  `@vizejs/marquette/schema`.
- The published adapter manifest schema is available from
  `@vizejs/marquette/adapter/schema`.
- Every optional contract field documents its default in JSDoc.
- Package builds enforce a 1 KiB gzip budget for authoring entries and 3 KiB
  or 4 KiB budgets for the validation entries.

Marquette describes components and environments; it does not author UI
components. Public Vize UI components use real `.vue` SFC files as their
canonical source, never authored `h()` calls or handwritten render functions.
