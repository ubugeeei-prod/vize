# Nuxt metadata dependency isolation

Issue: [#6880](https://github.com/ubugeeei-prod/vize/issues/6880).

## Actual consumer failure

The complete original VueFes run
[38132920869](https://github.com/ubugeeei-prod/vize/actions/runs/38132920869)
at `fd8b2d9da15a31ba17da0a53622e348fcf4debd8` failed the unchanged home
visual oracle at `0.1235105465271594`, against its `0.002` budget.
Its retained post-matcher CSSOM shows fourteen scoped headings at 18px/27px,
versus the reference's 20px/34px. The later global `:root h2` wins an equal
specificity tie with `h2[data-v-*]`. The CSSOM records later browser state;
it is not an atomic measurement at screenshot time.

Two separately authenticated response epochs establish the same eager path:
Nuxt entry → client plugins → router → generated routes → photo-page
`?macro=true` → `VFSection` → `VFHeading` and its style. The actual metadata
payload is only `{ prerender: true }`, but its native artifact retains the
unrelated component import. The App and lazy-layout modules have no static
path to that heading. The first epoch's response indices are
155/162/177/228/413/488/559; the reload's are
943/950/1007/1058/1241/1335/1392. The complete responses, status/headers,
and CSS hierarchy are retained with the failed run. Actual body refusals
remain incomplete observations; font binaries were not selected.

## Producer decision

Select dependencies only for `nuxt.definePageMeta`. Reuse the parsed OXC
program and resolved root-symbol references to distinguish imports from
property keys, callback parameters and local bindings. Keep selected whole
import statements in first-reference order. Follow referenced top-level
variable and function declarations transitively, emit those declarations in
authored order, and deduplicate declaration cycles with a worklist. Other
macro kinds keep their original import collection. Ordinary component code,
styles, maps, diagnostics, bindings and artifact source coordinates keep
their complete prior output.

This repairs the metadata producer. It does not sort styles, rewrite the
authored App, or alter visual expectations, thresholds, routes or deadlines.

## Independent oracle and corpus

The corpus retains all 2911 bytes of the original pinned photo SFC from
`vuejs-jp/vuefes-2025-website` at
`8eff3cd7dd79a7f47bbf2b78202b9aa2070ec5a6`, plus the complete before-source
compiler packet for five output shapes, Nuxt on/off and maps on/off. Its
genuine before-source test fails because unrelated imports remain. After
the repair, only the ten Nuxt artifact `moduleCode` fields change; every
other field in all twenty results compares exactly.

Eight whole SFC controls retain aliases, namespace/default imports,
selected whole import declarations, dependency order, local declaration
chains, shared dependencies, type-only references and lexical shadowing.
They use the actual Nuxt 4.0.3 `PageMetaPlugin`, after the actual Vue
3.6.0-alpha.2 full-script compilation. The oracle access copies the complete
pinned npm module and appends an explicitly recorded private plugin export;
all original module bytes and imports stay intact. Installed dependencies
and upstream repositories are unchanged. Both complete module bodies,
maps, statement spans and evaluated metadata/import traces are retained.

Seven controls have exact statement-body and runtime equality. The eighth
records a precise Nuxt limitation: its variable-initializer walk retains an
imported `input` despite a same-named arrow parameter. Native resolved
references omit that unused import; its entire module is asserted and the
metadata fields/calls agree, while both distinct import traces are kept.
The pinned oracle also actually refuses mutually dependent functions with
`RangeError`; a native regression retains both declarations without
recursing during extraction. Neither limit is reported as oracle parity.

The existing 778 SFC library tests, original 60-case mounted/SSR page-meta
runtime differential and three new native tests pass locally. Earlier
local failures from the macOS temporary-directory alias and a missing
private dependency link are preserved; canonical temporary paths and the
existing protected dependency target resolve those prerequisites. These
local results do not qualify the hosted original 38-case visual matrix.

## Delivery boundary

The composed source must pass the original full hosted matrix before queue
admission. Reuse its existing single native build and existing metadata
contract step for the new native/oracle dependency consumer. Preserve the
failed run and its full pixels. Actual source-bound consumer and release
acceptance remain required; no earlier fixture green qualifies the failed
application.

The fd8 archive lacks every selected hidden formatter-history receipt,
raw Cargo stream and frozen binding: the actual upload used
`include-hidden-files: false`. Its two retained emitted/local ELF files do
not restore those missing producer records. Enable hidden files in the same
existing upload, retain its exact selected JSON/bin/diff/node paths, and
assert that source policy. The next actual archive must establish complete
custody; the policy test alone does not prove runtime upload membership.
