# Original primitive string value facts

Issues: [#6838](https://github.com/ubugeeei-prod/vize/issues/6838),
[#6839](https://github.com/ubugeeei-prod/vize/issues/6839) and
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

Status: independent source review is clear; exact hosted source and protected
acceptance remain pending. The future primitive setup Vapor consumer has no native
runtime acceptance from these primary-only controls.

## Genuine source fact

The existing original variable declaration event classifies its actual
`Expression::StringLiteral` once. The original OXC `lone_surrogates` flag records
`PrimitiveStringWithLoneSurrogates`; otherwise one decoded value byte scan for
NUL or carriage return records `PrimitiveStringWithNulOrCr`. Other original
primitive literals keep `PrimitiveLiteral`. A string containing both a lone
surrogate and NUL/CR takes the lone-surrogate class; both classes remain primitive.

This refines the existing unit enum. Declaration fields, ScriptUnit, tables,
row counts and allocation remain unchanged. No initializer AST/value storage,
source search, reparse, recovery AST walk, observer capability or target policy
is introduced. The row is minted by the original binding producer with the
same File, BindingId, ScriptUnit and current scope as before.

`InitializerKind::is_primitive()` includes all three original primitive classes.
The common setup eligibility and TypeScript annotation guard, existing Vue
exposure read classifier, genuine native selected setup classifier, native
For collection classifier and the actually merged original setup SSR const
classifier use that predicate. Existing const/let/var behavior,
DOM output and annotations remain accepted. Common classifications do not prove
an initializer is numeric, a collection, or safe for a particular target.

The future new Vapor target may conservatively refuse the two hazardous classes
at its genuine read admission, and inspect the actual direct Scalar StringLiteral
value/flag separately. This provider adds no such target policy. Generic source
completion and common DOM acceptance do not grant Vapor hydration authority.

## Actual primary normalization evidence

The primary graph is actual plugin-vue `6.0.7` and Vue compiler/SSR/runtime
`3.6.0-rc.9`. Every original whole SFC is retained unchanged; actual stock inline
CSR and SSR default modules, full graph code/maps and helper modules are captured.
Chromium `152.0.0.0` consumes their exact loaded defaults through the actual
development and production Vapor runtimes.

Seven originals cover const `a\0b`, let `a\u0000b`, const `a\rb`, let `a\r\nb`,
direct root `a\0b` and `a\rb`, and a const LF control. Both runtime flavors run
mount and hydration, for 28 observations. Real Chromium HTML parsing removes
NUL and normalizes CR/CRLF to LF. All six hazardous hydrations per flavor retain
their original Text node but the real runtime changes its data back to the
original JavaScript value. Development reports six hydration mismatch warnings;
production reports none despite the same mutations. LF has no mutation/warning.
All fourteen hydrated original nodes survive and every unmount leaves zero nodes.

Five additional originals cover const `a\ud800b`, let `a\udc00b`, a direct
scalar lone high surrogate, an authored valid surrogate pair and explicit U+FFFD.
The unchanged actual SSR result is sent in a physical UTF-8 HTTP response.
Chromium parses lone units as U+FFFD, then the real runtime changes the retained
Text data to the original lone unit. Development has three mismatch warnings;
production has none. Valid-pair and U+FFFD controls have identical original and
mounted trees. Ten original hydrated nodes survive and all twenty mounts clean
to zero. No source transport or initializer value is inferred from a coarse tag.

happy-dom falsely preserves NUL/CR in `innerHTML`. A JavaScript `innerHTML`
assignment also bypasses physical UTF-8 replacement of lone units. Those earlier
primary processes are retained separately and grant no real parser/transport
equivalence credit. Warning absence and retained-node identity alone cannot
validate an unchanged hydrated tree.

Exact primary-only evidence retained at
`/tmp/vize-vapor-setup-normalization-20261004` and
`/tmp/vize-vapor-setup-surrogate-20261004` includes original sources, complete
graphs/maps, raw child stdin/stdout/stderr/status/signal/error, browser snapshots,
observations and the loader/HTTP recipes. The browser only rewrites genuine
parsed module-source ranges; authored comments/string contents remain unchanged.

Receipt SHA256 values:

- NUL/CR: `7117dcf82e340ebeb5ac65a20cb363b6b990fcb9a24b1e7570a3eb2364d1be8a`.
- Lone units/UTF-8: `2051460cf4818b809a85e7e242d9d03e25751996f6f19f744e65d237fafdd7c9`.

## Required laws and unfinished work

Genuine normally owned NativeSelectedSetup laws cover JS/TS const/let/var,
multiple authored escape spellings, harmless escaped backslashes, LF, a valid
surrogate pair, U+FFFD, original unit/scope/binding membership and annotation
custody. Existing Vue exposure, genuine selected DOM/SSR and original For classifiers
retain their primitive const/mutable semantics. An independent original enum and
Declaration field-layout law compares size/alignment without loosening budgets.
Existing whole DOM captures remain unconditional and unchanged.

Source and full protected instruction/allocation/complexity/corpus gates must
accept exact source and actual queue candidates. No budget or gate is relaxed.
The genuine same-Writer runtime-segment provider #7687 actually merged as
`c40b55bee6560bfc842f1f132115c72059549f70`; it is an independent prerequisite.
The new Vapor target still needs typed hazardous-string refusals, complete native
modules/maps and exact hosted
mount/hydration/update-or-refusal/zero-cleanup captures. Compiler fix history
[#6880](https://github.com/ubugeeei-prod/vize/issues/6880), general strings/setup,
events/control/nested semantics and default replacement remain unfinished.

## First source failure and actual common replay

Exact `ddba36b4a2` Check `37166832677` built successfully, then Rust shard 1
ran 3,817 tests with one failure in the existing legal strict-literal law. Its
actual `\0` input still correctly gains setup admission, but the law fixed every
initializer to the old `PrimitiveLiteral` tag. The same original per-literal table
now asserts `PrimitiveStringWithNulOrCr` for that one input and exact ordinary
primitive tags for its other seven inputs; all original owner/admission assertions
remain. The failed raw log is retained at
`/tmp/vize-primitive-string-ddba-rust1.log`; that source was never queued.

Original setup SSR provider #7684 actually merged as `6a3f6e779a65` after that
source was authored. The real-main replay preserves its original Const read
classification with `is_primitive()` and adds genuine same-owner DOM/SSR
BindingId, declaration class and const/mutable read equality laws. No unmerged
SSR target source or artificial target dependency is imported. Fresh exact source
and full protected acceptance remain required.

## First protected composition and exact inherited attribution

Exact corrected source `a77ef92b89` Check `37167993830` and all four required
contexts succeeded. Genuine 96 JS/TS source classes, 21 same-owner DOM/SSR reads,
legal strict literal and original enum/Declaration layout laws passed. The
independent PR entered candidate `db719dc18a09` / Check `37168519585` after
clean actual-main and healthy-tail unions. Its native 100 measurements and
ratchets held, but the four formatter ceilings failed:

| Formatter        | Actual | Fixed ceiling |
| ---------------- | -----: | ------------: |
| simple SFC       | 293896 |        293575 |
| reused SFC       | 275027 |        274723 |
| large script     | 929114 |        929044 |
| complex template | 244583 |        244347 |

The owned PR was promptly drafted and dequeued, including the raced fresh
`b2bb7ed7` admission; its then-current queue and auto-merge were confirmed null. The failed raw job is retained
at `/tmp/vize-primitive-string-db719-instruction-failure.log`. No red candidate
or failed formatter measurement earns protected acceptance.

The actual preceding candidate `20d07392b300` / Check `37168403389` already failed
those same four ceilings before this provider. Its three complete formatter
run JSON payloads are byte-identical to all three owned-candidate payloads,
SHA256 `47244fdabf53df44ff1f78c52c7dff3abb6f9ff335342e45d33bd1069f5cf599`.
Inputs and measurement methodology agree. Binary hashes differ (`5adf807e` versus
`46514494`), so binary identity is not asserted. Glyph's normal dependencies are
L0/L1, and this provider changes L2/L3; the observed failure predates its delta.
Full tiny profiles and attribution receipt are retained at
`/tmp/vize-primitive-string-a77-preflight-20261004/formatter-predecessor-attribution.json`.

The failed prospective prefix included `f60de62e`; after its withdrawal, the
independent healthy Canon candidate `1d91` passed all 104 fixed measurements.
One new genuine owned candidate excluding that failed prefix is authorized after
fresh actual-main/tail preflight and fresh exact source proof if its head changes.
The held private `4aee` replay recorded this attribution but was not pushed, preserving
the already healthy exact `a77` admission instead of invalidating its proof. No
formatter source, calibration, instruction boundary, threshold, headroom or budget
changed; the unrelated formatter optimization remained independent.

## Literal protected acceptance

The one fresh healthy candidate `154136bb049dc57252d3a5090687bd58caca87db`
actually merged #7689 at `2026-10-04T02:08:56Z`, with a valid verified signature.
Exact source remains `a77ef92b89`; protected Check `37169188504`, Nuxt
`37169188157` and Musea `37169188134` are terminal success. All four full Rust
shards pass (3856 + 3784 + 3714 + 3922 = 15,276), with no failures or skips.
All 100 native and four formatter measurements pass their unchanged ceilings in
three repetitions. Genuine 96 source classes, 21 same-owner DOM/SSR reads,
legal literal, comment/value, original enum/Declaration layout and For preservation
laws each pass in the fresh protected artifacts.

The mandatory existing Vapor capture contains twelve standalone complete modules
and maps with 26 configurations/23 hydrations, plus eleven scriptless whole modules
and maps with 23 configurations/22 hydrations/58 retained original descendants.
Actual loaded default-component identity and zero unmount residue hold. The packet
is retained at `/tmp/vize-primitive-string-154-acceptance-20261004/receipt.json`,
SHA256 `ef955ed7ea728167f91c07758d91e30032dc02249ae33a48accb476dbaaffce4`.
These observations close this neutral provider only. The primitive setup Vapor
component is not emitted by #7689; its separate L3 provider and genuine L4 consumer
still require source, whole-module/map/runtime and protected acceptance.
