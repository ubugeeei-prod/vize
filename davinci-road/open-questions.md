# Davinci — Open Questions

> [!NOTE]
> Active design discussions. Each entry gets a decision record (moved to the
> [charter](./README.md#decided-positions)) or is dropped with a note. Decided
> entries become stubs pointing at their charter row — never silently deleted.

## Decided (stubs)

- **Naming** → charter #11. Stage aliases are the primary implementation names
  (`vize_s1`, `vize_s2`, `vize_s3`, `vize_s1_to_s2`). S1 has already been
  mechanically renamed from Sinopia; S3 currently keeps the `vize_impeto`
  package id; remaining art names stay courtesy aliases and, where not yet
  renamed, historical package ids until their own rename PRs land.
- **S3 scope** → charter #9. DOM + Vapor through S3; SSR thin S2→S4 path
  reading partition facts. Phase 3 measurements keep veto power.
- **Incrementality** → charter #10. salsa in the resident tier only; fused
  non-salsa pipeline for one-shot CLI; block content keys as firewall queries.
- **Fact query API** → charter #8. Static demand declarations + debug-build
  undeclared-access detector.
- **pug fidelity** → charter #12. First-class S1 dialect.
- **SFC style coordination** → charter #13. `v-bind()` bindings visible as S2
  ops.
- **Foreign expression type checking** → charter #14. Projection duty lives in
  the expression-dialect contract; boundary-typed integration is the fallback
  tier, not the default.
- **Contract linking** → charter #15. Two tiers: compiled-in traits + features
  for first-party, out-of-process serialized contract for external.
- **DevTool protocol** → [devtool.md §Transport](./devtool.md#transport)
  (P2-19 spike, [record](./plan/phase-2-records/p2-19.md)). Document over
  JSON-RPC: the P2-18 feed document is the unit everywhere; C-7's server
  speaks content-mapper-style JSON-RPC with `initialize` negotiating
  `schema_version` before any payload; served files stay the at-rest form,
  the wasm playground keeps the embedding; JSON-lines rejected.
- **Fusion depth for the build path** →
  [P2-12b record](./plan/phase-2-records/p2-12b.md). Source-map-free DOM
  compiles report one observer-facing S2 build walk. Synthesized and preserving
  facts fold before codegen; Vue 2 legacy sugar remains a compatibility pass,
  and SSR/Vapor fusion waits for phase 3.

## Orphan analyses: productize or cut

`RaceConditionTracker` and `ProvideInjectTracker` had zero consumers at the
2026-08-13 audit; `EffectGraph` had one (Doctor). Every product that doesn't
earn corpus trust gets its fact group demand-gated to zero cost rather than
deleted, per charter #5.

**Re-measured at the phase-4 re-cut (2026-09-21):** provide/inject and race
analysis are now read by `vize_croquis_cf`'s cross-file rules, which `vize lint`
and Doctor enable; `EffectGraph` feeds Doctor and the croquis_cf complexity
report. The zero-consumer products are now `Croquis.hoists`, `Croquis.symbols`,
`Croquis.used_directives` and the `reactivity_overlay` family
([consumption matrix](./plan/croquis-consumption.md)). Owners:
[P4-4a](./plan/phase-4-tasks.md#p4-4a--orphan-verdicts-for-non-effect-products)
rules on those and on the corpus soundness of the provide/inject and race rules;
[P4-4b](./plan/phase-4-tasks.md#p4-4b--effect-graph-verdict) owns `EffectGraph`
once P3-6 gives it a Vapor consumer.

## Rule-corpus fairness measurement

Charter #7 needs a metric: which Patina rules are neutral-core (should run on
SFC + JSX + external dialects), which are Vue-dialect-bound (`v-model`
modifiers), and which are container-bound (SFC block structure)? The phase-0
rule-parity matrix defines the classification (today 248 rules: 92 / 133 / 23);
phase 4's exit gate consumes it.

**Recommendation (phase-4 re-cut, 2026-09-21):** derive the class from the
fact groups and op kinds a rule demands (static demand declarations, charter #8,
make it nearly free) and keep the existing
[`rule-parity-overrides.toml`](./plan/rule-parity-overrides.toml) sidecar as the
only override path, each override carrying a reason; no per-rule
`dialect_scope` field. Owner:
[P4-8a](./plan/phase-4-tasks-later.md#p4-8a--neutral-core-rule-wave) confirms or
amends this in its first installment, after which this entry becomes a stub.

## Complexity metric definition

Which metric family for template CFGs — cyclomatic, cognitive, or both — and
how cross-file attribution works (does a complex child component tax its
parents, or only its own score?), plus thresholds and rule presentation. The
existing scorer (`vize_croquis_cf/src/rules/complexity.rs`) counts `v-if`s,
`v-for`s, components and logical operators scanned from expression text; it has
no CFG and sums a whole project.

**Recommendation (phase-4 re-cut, 2026-09-21):** both families, computed per
component over S2 regions. **Own cyclomatic** = 1 + decisions (each `ui.if`
branch beyond the first, one more for an `ui.if` without `v-else`, each
`ui.for`, each `&&`/`||`/`??`/`?:` in a retained expression AST; opaque
expressions add nothing and are counted as unknown). **Own cognitive** follows
the SonarSource model: +1 per structure, plus the nesting depth of nested
`ui.if`/`ui.for`/scoped-slot regions, +1 per run of like logical operators.
**Attribution:** a child never taxes its parent's _own_ score — the lint rule
fires on own complexity only, so composing components never makes a parent
look worse — while a separate **rendered** complexity (own + Σ own over the
distinct child components reachable in the render tree, strongly connected
components collapsed) is the cross-file number charter #17 asks for and drives
Doctor hotspots. **Presentation:** one warning-severity rule with tier `exact`;
**thresholds are deferred to corpus data**: the default is pinned at the
measured corpus p95 by
[P4-9a](./plan/phase-4-tasks-later.md#p4-9a--template-cfg-complexity-facts-and-metric-spec),
which writes the metric spec and confirms or amends this recommendation;
[P4-9b](./plan/phase-4-tasks-later.md#p4-9b--cross-file-complexity-rule-and-doctor-finding)
ships the rule and the Doctor finding.

## `no_std` boundary reality check

Davinci-owned crates are `no_std + alloc` by charter #18, but the practical
boundary depends on dependencies: oxc crates and lightningcss assume `std` in
places, salsa is resident-tier-only anyway, and rayon is a `std` feature.
Needs an early audit: which existing crates could honestly become `no_std`,
what the `wasm32-wasip2` CI target covers (core compile lane vs full CLI), and
whether the WASI component model doubles as the out-of-process contract
transport (charter #15) — evaluate against the prior-art findings.

## AI optimization loop guardrails

**Decided** → charter #32: optimization PRs auto-merge on full gate passage;
contract/semantics changes stay maintainer-reviewed. Remaining detail for the
implementation roadmap: the sandboxing of optimization experiments (worktree
isolation, corpus-run quotas) and the audit trail format for auto-merged PRs.

## JS plugin API shape

**Decided by measurement (P4-16, 2026-09-22)** →
[P4-16 record](./plan/phase-4-records/p4-16.md): serialized S2 visit batches
over sync napi; fact demands declared as a static list in the plugin
manifest; content keys over the plugin's version, code fingerprint, batch
schema and file; the batch doubles as the WASM-tier contract. The measured
proxy arm was not slower on small templates and was rejected on portability,
write-back and hashability grounds (numbers in the record). ESLint
compatibility stays a non-goal.

## App-level fact provider contract

Route trees, `definePageMeta`, i18n catalogs: in-tree providers cover Vue
Router and Nuxt (Vue-family scope), but the provider interface should be the
same one external ecosystems would use. Open: is a convention provider a third
kind of first-party plug-in (like input dialects), or a consumer of the
cross-file fact API with write access?

**Recommendation (phase-4 re-cut, 2026-09-21):** neither a new plug-in kind nor
general write access. A provider is a **project-level population pass in the
fact demand graph**: it declares its ambient inputs (the files, globs and
config it reads — exactly what P5-1's key manifests need) and **exclusively
owns** its output groups (single writer, enforced by the registry), and
consumers demand those groups like any other fact. In-tree providers compile in
behind cargo features (charter #15's first tier); external providers use the
same interface through charter #29's "custom fact providers" hook family.
Owner: [P4-10a](./plan/phase-4-tasks-last.md#p4-10a--provider-contract-and-vue-router-provider)
lands the contract with the Vue Router provider and turns this entry into a stub.

## Communications (charter #45) and the Vue Fes Japan 2026 presentation

Charter #45 keeps Davinci repository-internal "until the maintainer chooses".
On 2026-09-21 the maintainer set a public presentation target — Vue Fes Japan
2026 on 2026-10-24 — and a completion target of 2026-09-23, which is why
[phase 4 was re-cut early](./plan/phase-4.md). This entry records what changed,
as the charter's revisit rule requires; amending row #45 itself is the
maintainer's decision (P6-12 was its planned decision point).
