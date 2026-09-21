# Phase 4 — Task contracts, P4-10a through P4-17

> [!NOTE]
> Continuation of [phase-4-tasks-later.md](./phase-4-tasks-later.md) under the 350-line source budget. Same authority, same format; the TODO index in [phase-4.md](./phase-4.md) links each task to whichever file holds its contract.

## P4-10a — Provider contract and Vue Router provider

**Start gate:** startable now — P3-independent.

**Lane:** H

**Deliverable:** the app-level provider contract recommended in [open questions](../open-questions.md#app-level-fact-provider-contract) — a provider is a project-level population pass in the P4-1 demand graph that declares its ambient inputs (files, globs, config it reads; the future P5-1 key manifest) and **exclusively owns** its output groups — plus the in-tree `VueRouterProvider` producing `RouteTree` and `RouteParams` from statically analyzable `createRouter({ routes })` records, with route-typing diagnostics at `router.push({ name, params })` and `<RouterLink :to>`.

**Steps:**

- [ ] `crates/vize_croquis_cf/src/providers.rs` (the `Provider` trait: `const INPUTS`, `const OUTPUTS: Demand`, single-writer check in the registry) and `providers/vue_router.rs`
- [ ] Diagnostics for an unknown route name and missing or extra params; tier `sound` within the declared domain (static route records, literal names)
- [ ] Maestro's `crates/vize_maestro/src/ide/ecosystem/router.rs` (687 lines) completions read the provider groups

**Acceptance:** exact diagnostic snapshots over new fixtures `tests/_fixtures/davinci-routes/` (TS-9); a second writer to `RouteTree` is rejected by the registry with its exact error; TS-35; Maestro router completion tests exact; the open-questions entry becomes a stub pointing at the record.

**Deps:** P4-1a, P4-2, P4-6a.

**Non-goals:** Nuxt (P4-10b); i18n catalog providers; external providers (the JS fact-provider hook family, P6-7).

## P4-10b — Nuxt provider and projected route types

**Start gate:** startable now — P3-independent (waits behind P4-5b).

**Lane:** H

**Deliverable:** `NuxtProvider` — the `pages/` filesystem route tree plus `definePageMeta` shape and reachability — generalizing the route and page parts of `crates/vize/src/commands/check/nuxt/` (4,916 lines with tests), and typed route params flowing into the P4-5b projection.

**Steps:**

- [ ] `crates/vize_croquis_cf/src/providers/nuxt.rs`; `check/nuxt` reads provider groups instead of rescanning
- [ ] Dead-route and invalid-`definePageMeta` diagnostics; projection emits route-param types for `useRoute()`/`navigateTo`

**Acceptance:** TS-40 fixtures gain route-param records, compared exactly; Nuxt check fixtures unchanged (TS-9); dead-route fixtures exact; TS-35.

**Deps:** P4-10a, P4-5b.

**Non-goals:** Nuxt module auto-import changes; runtime route validation.

## P4-11a — Content-model tables and exact per-file checker

**Start gate:** startable now — P3-independent.

**Lane:** I

**Deliverable:** a spec-derived HTML content-model table committed as generated data — categories (flow, phrasing, interactive, …), permitted content, void elements, implied end tags, table anatomy — produced from a pinned WHATWG snapshot by `rust-script tools/commands/davinci/html-content-model.rs --write` with a `--check` staleness mode, read by `vue/permitted-contents` and the nesting checks in `rules/html/`; tier `exact` (charter #36's provably total checker target).

**Steps:**

- [x] `crates/vize_patina/src/html_content_model/` (table + total checker); the generator and its `--check` wired into `tests/tooling/davinci-matrices.test.ts` (TS-12)
- [x] Seeded-defect classes for nesting violations added to `tools/commands/davinci/seed-defects.rs`

**Acceptance:** `--check` proven to fail on an injected table edit; TS-37 nesting classes at 100% recall, count-exact by identity; TS-38 zero untriaged candidates for the upgraded rules on the corpus shard; TS-9 changes listed line by line in the PR.

**Deps:** P4-6a.

**Non-goals:** cross-component composition (P4-11b); ARIA rules (P4-8a).

**Landed 2026-09-22:** the exact checker: 21 classes, verdicts pinned against Chromium and parse5. The table is generated from the pinned WHATWG snapshot, and `--check` is proven against an injected edit. TS-37 is 100% by identity on the miniature set, the corpus shard and five apps. TS-38 is triaged (FP-2), and Lean proves the walk sound. See the [P4-11a record](./phase-4-records/p4-11a.md).

## P4-11b — Composed cross-component conformance

**Start gate:** startable now — P3-independent.

**Lane:** I

**Deliverable:** the check no per-file tool can do — `<p><MyCard /></p>` where `MyCard`'s root is a `<div>` — from a `ComponentRoots` group (a child's possible root elements from its S2 page: one element, `ui.if` alternatives, fragment children; a `<slot>` pass-through is unknown) joined with the parent's insertion content model. The rule `html/cross-component-nesting` reports with a witness naming the parent element span, the child root span and the resolved component identity; tier `exact` within the declared domain (statically resolved components, no dynamic `:is`); `unknown` produces silence.

**Steps:**

- [ ] `crates/vize_patina/src/html_content_model/composed.rs` over P4-3b's render-tree identities
- [ ] A seeded cross-component defect class; fixtures for aliased imports, same-basename components, `v-if` multi-root, fragment roots, slot pass-through

**Acceptance:** TS-37 cross-component class at 100% recall; TS-36 every witness verifies; TS-38 zero untriaged on the corpus; fixture snapshots exact (TS-9).

**Deps:** P4-11a, P4-3b, P4-6b.

**Non-goals:** CSS-driven layout checks; dynamic components.

## P4-12a — Style specification

**Start gate:** startable now — P3-independent.

**Lane:** J

**Deliverable:** `davinci-road/plan/style-spec.md`, written from charter #41's blank-slate discussion: every decision a numbered rule with a fixture pair `crates/vize_glyph/tests/style_spec/<rule-id>/{input,output}.vue` (the TS-41 set).

**Steps:**

- [ ] Draft the rules, one fixture pair each
- [ ] `tests/tooling/davinci-style-spec.test.ts`: every rule id has a fixture pair and every pair a rule

**Acceptance:** the spec/fixture bijection test green and proven to fail on an orphan fixture. **Review point:** the maintainer signs off the style direction — no direction is pre-committed (charter #41).

**Deps:** none (phase-2 exit).

**Non-goals:** implementing the style (P4-12b); pug layout (P4-12c).

## P4-12b — Glyph on S1

**Start gate:** startable now — P3-independent.

**Lane:** J

**Deliverable:** Glyph's template formatting reimplemented over `vize_s1` trees, deleting the byte scanner `crates/vize_glyph/src/template/formatter.rs` (597 lines) and its `formatter/` submodules; SFC blocks through the `vize_croquis` splitter; script formatting through `oxc_formatter` unchanged.

**Steps:**

- [ ] Rewrite `crates/vize_glyph/src/template*` on S1; `cargo test -p vize_glyph --test style_spec` runs the TS-41 pairs
- [ ] Attach the churn-vs-old report to the PR (reported, not gated)

**Acceptance:** TS-5 — idempotence, parse-preservation, lint-agreement and pug — with an empty waiver ledger; TS-41 exact; `grep -rn "memchr" crates/vize_glyph/src/template` empty; `davinci-glyph-stage-alias.test.ts` extended for the S1 edge.

**Deps:** P4-12a.

**Non-goals:** pug formatting (P4-12c); an OXC lossless script wrapper.

## P4-12c — Pug as an S1 dialect

**Start gate:** startable now — P3-independent.

**Lane:** K

**Deliverable:** charter #12 made real: a pug parser producing the lossless S1 surface with `Unexpected`/`Missing` holes (`crates/vize_s1/src/pug/`), an S1→S2 lowering (`crates/vize_s1_to_s2/src/lower/pug*`) so compile, lint, format and type-check share the lanes, and pug formatting in Glyph.

**Steps:**

- [x] Parser with `render(parse(src)) == src` fidelity over the corpus pug templates and a malformed set
- [x] Lowering total over the battery and every truncation (the P2-8 shape)
- [ ] Compile oracle: compiling a pug SFC equals compiling the same SFC with its template replaced by the pinned `pug` package's HTML rendering, byte for byte

**Acceptance:** TS-19 and TS-20 extended to pug and green; the compile oracle exact over the corpus pug SFCs with scope proof; TS-5's pug property green.

**Deps:** none (phase-2 exit).

**Non-goals:** pug-specific lint rules; pug mixins beyond Vue's documented support.

_Slice 1 2026-09-22 (surface, lowering, compile lanes):_ see [P4-12c record](./phase-4-records/p4-12c.md)

## P4-13 — Musea onto S0 and S1

**Start gate:** startable now — P3-independent.

**Lane:** L

**Deliverable:** `crates/vize_musea/src/parse.rs` and `parse/` (1,232 lines of `memchr` hand scanning) replaced by the `vize_croquis` SFC block splitter plus S1 template trees for the `<art>`/`<variant>` custom blocks, with `ArtDescriptor` unchanged.

**Steps:**

- [x] Re-implement `parse_art` over the splitter and `vize_s1`; delete the scanner

**Acceptance:** `cargo test -p vize_musea` snapshots unchanged (TS-1); TS-11 art surface empty; `grep -rn "memchr" crates/vize_musea/src/parse*` empty; the six Musea rules' fixtures unchanged (TS-9).

**Deps:** none (phase-2 exit).

**Non-goals:** new Art features; Musea UI.

**Landed 2026-09-22:** `parse_art` on the splitter and S1, the scanner deleted after the differential lane, the corpus frozen as a fingerprint golden — see the [P4-13 record](./phase-4-records/p4-13.md) for the lane results and the divergence ledger.

## P4-14a — Structured diagnostic renderer

**Landed 2026-09-22** — full record: [phase-4-records/p4-14a.md](./phase-4-records/p4-14a.md).

**Start gate:** startable now — P3-independent.

**Lane:** M

**Deliverable:** a rustc/Elm-grade renderer over `vize_davinci::Diagnostic` in `crates/vize_davinci/src/render.rs`: `error[code]: message`, `file:line:col`, a source excerpt with primary `^^^` and secondary `---` labels, help/note/suggestion parts with diff-style fixes, color and no-color, line/column derived at render time from the S0 line index (P2-1's contract), messages resolved through a caller-supplied `Catalog` generic (static dispatch; `vize_carton::i18n::Translator` implements it at the CLI edge), exposed as `vize lint --format rich`.

**Steps:**

- [x] Renderer + `Catalog` trait (`#![no_std]` + `alloc`); east-asian-wide width handling for ja/zh excerpts
- [x] Register **TS-53** in [test-suites.md](./test-suites.md): `cargo test -p vize_davinci --test diagnostic_render` + `node --test tests/tooling/davinci-diagnostic-catalog.test.ts`

**Acceptance:** TS-53 snapshot fixtures exact per locale (en/ja/zh), including wide-character alignment and multi-label excerpts; TS-24; TS-1, TS-13.

**Deps:** none (phase-2 exit).

**Non-goals:** catalog coverage (P4-14b); `--explain` (P4-14c); LSP presentation.

## P4-14b — Catalog completeness for every producer

**Start gate:** startable now — P3-independent.

**Lane:** M

**Deliverable:** every diagnostic code has en/ja/zh entries — the 56 compiler `ErrorCode`s, all 248 rules (121 lack a `description` today), Canon and croquis_cf codes, S2 verifier codes — with producers routed onto the unified channel (a `CompilerError` → `Diagnostic` adapter) and the catalog moved out of the over-budget JSON files into per-producer tables.

**Steps:**

- [ ] `tests/tooling/davinci-diagnostic-catalog.test.ts` enumerates codes mechanically from the producers and fails on any missing locale
- [ ] Fill the gaps; English messages unchanged

**Acceptance:** the catalog test green and proven to fail on a removed entry; 0 of 248 rules without a description in any locale; TS-9 English snapshots unchanged; TS-53.

**Deps:** P4-14a, P4-6c.

**Non-goals:** locales beyond en/ja/zh.

## P4-14c — Explain pages and witness why

**Start gate:** startable now — P3-independent.

**Lane:** M

**Deliverable:** `vize explain <code>` (new `crates/vize/src/commands/explain.rs` + a `cli.rs` variant) generating pages from rule metadata — description, help, tier, declared domain, fixture examples — and witness-derived "why" notes in the renderer (`note: because …`, one line per `WitnessLink`).

**Steps:**

- [ ] Page generator over `RuleContract` and the catalog; renderer expansion over `Witness`

**Acceptance:** TS-53 explain snapshots exact per locale for every code (the list generated, not hand-written); witness-why snapshots for the P4-3c and P4-11b witnesses.

**Deps:** P4-14b, P4-6b.

**Non-goals:** a docs website.

## P4-15a — Seeded-defect matrix at full scale

**Start gate:** startable now — P3-independent.

**Lane:** N

**Deliverable:** `tools/commands/davinci/seed-defects.rs` extended so every rule with tier `exact` or `sound` has a generated defect class (manifest: file, span, expected rule id), asserted by identity.

**Steps:**

- [ ] One generator per class; classes enumerated from `rule_contracts.rs`, so a new exact/sound rule without a class fails
- [ ] Misses triaged in [ledger-fn.md](./ledger-fn.md)

**Acceptance:** TS-37 100% recall per in-domain class with scope proof; `tests/tooling/davinci-fpfn-pilots.test.ts` extended; no untriaged FN entry.

**Deps:** P4-6c.

**Non-goals:** heuristic-tier rules (no recall claim).

## P4-15b — Corpus suppression triage to zero untriaged

**Start gate:** startable now — P3-independent.

**Lane:** N

**Deliverable:** suppression telemetry over the full 142-project corpus on a weekly CI schedule, with every candidate triaged `fixed` or `justified-with-witness` in [ledger-fp.md](./ledger-fp.md).

**Steps:**

- [ ] Weekly workflow running `rust-script tools/commands/davinci/suppression-telemetry.rs` over the hydrated corpus; mapped-rule table extended for migrated rules

**Acceptance:** TS-38 zero untriaged candidates with full-corpus scope proof; the workflow pinned by a tooling test.

**Deps:** P4-15a, P4-8c.

**Non-goals:** user telemetry collection.

## P4-16 — JS plugin SDK spike

**Start gate:** startable now — P3-independent.

**Lane:** O

**Deliverable:** the API-shape decision of [open questions](../open-questions.md#js-plugin-api-shape) — serialized visit batches vs proxies, worker vs sync napi, JS-side demand declaration — proven with one real custom rule over the S2 facade through `vize_vitrine`'s napi lane with batched node visits.

**Steps:**

- [ ] `crates/vize_vitrine/src/napi/plugin*` spike and one rule; measure batch vs proxy cost
- [ ] Record the decision; the open-questions entry becomes a stub

**Acceptance:** the spike rule's output byte-identical across two runs and its time attributed per plugin in lint output, in a node test; decision recorded; the spike code kept with tests or deleted, and the PR says which (GA is P6-7).

**Deps:** P4-1a, P4-7a.

**Non-goals:** caching (P5-13); the other three hook families (P6-7).

## P4-17 — Phase exit

**Start gate:** gated on P3-16 — phase order.

**Lane:** X

**Deliverable:** the exit gate in [phase-4.md](./phase-4.md) evaluated inline — a line is ticked only when satisfied, an unticked line names its blocker, no line is softened — with the in-phase old paths deleted or recorded as unfinished deletions with owners (charter #26) and the `davinci-differential` lanes' retirement restated.

**Steps:**

- [ ] Evaluate every exit-gate line with evidence; delete old paths; C-14 corpus audit for the surfaces touched (pug, JSX lint, projection)

**Acceptance:** every exit-gate line ticked with evidence or carrying a named blocker.

**Deps:** every other phase-4 task, P3-16.

**Non-goals:** re-cutting phase 5 (its own task).
