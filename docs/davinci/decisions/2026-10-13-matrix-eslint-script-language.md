# Respect TypeScript script language in the reference lint parser

Issues: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881),
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Real Project Matrix [38331707741](https://github.com/ubugeeei-prod/vize/actions/runs/38331707741)
checked immutable source `2817a0aa13518dd46c16bc8bf56ac831f68c24dc`.
Its original lint summaries report eight reference parse failures across six
projects: gui-for-singbox, hoppscotch, inertia, uni-app, vitepress and vux.
The [whole six relevant summaries and ZIP/member custody](./fixtures/matrix-eslint-script-language/matrix-custody.json)
are retained without replacing the original counts with replay observations.
These summaries did not retain fatal diagnostic bodies or affected paths.

## Cause and change

Both the actual Rust runner's embedded JavaScript and the compatibility baseline
set `ecmaFeatures.jsx = true` for every `.vue` script. The pinned TypeScript parser
uses this setting for unknown extensions, including `.vue`, so valid
`<script lang="ts">` is interpreted as TSX. This rejects a generic arrow in
GUI.for.SingBox and angle-bracket type assertions in Hoppscotch and Inertia.

Use vue-eslint-parser's script-language parser selection. The `ts` adapter calls
the same pinned TypeScript parser with JSX disabled. JS/default, JSX and TSX use
the original parser and options. No package pin, rule selection, severity,
processor, suppression handling, comparable surface, or FP/FN budget changes.

Both report producers now retain complete `baseline.parseErrorResults` before
classification discards rule-less findings. These preserve source and all ESLint
result fields, including fatal messages and suppressed messages. Parse errors
continue to make the reference unusable; retaining them supplies evidence for
repair rather than a passing verdict.

## Reproduction and verification

The differential corpus at
`tests/_fixtures/differential/lint/reference-script-language/` retains all eight
whole pinned inputs, repository/revision/license identities, source digests,
complete replayed fatal packets, the frozen original producer script and its
hash, and the original H2817 rule-selection vector. Fatal packets were replayed
with the pinned providers; they are not claimed to be recovered original Matrix
raw packets.

`node --test tests/tooling/lint-baseline-script-language.test.mjs` runs the actual
candidate Rust producer's embedded script and the compatibility baseline. Both
must match complete independently configured public ESLint results. The frozen
original script must reproduce each complete retained fatal packet. Actual JSX
and TSX controls preserve a real unsuppressed Vue finding, and malformed TS stays
strictly unusable. Existing processor/directive controls also remain passing.

A separate [whole six-project replay](./fixtures/matrix-eslint-script-language/six-project-replay.json)
uses current source b45d789b317933d48dadbba4841c8e353231547b's mapped rule vector
and 1,392 unchanged fixture inputs with the registry's existing ignore/glob
selection. Complete before/after ESLint results change for exactly three files,
all repaired TypeScript inputs. The other 1,389 whole results are unchanged.
This baseline-only replay is not a complete Matrix or candidate lint parity run.

## Remaining failures

Five reference failures remain byte-exact and unusable:

- Three uni-app iOS `lang="uts"` files use native named-argument syntax,
  for example `setTitle(newValue, for = UIControl.State.normal)`.
- VitePress's generator template starts with an unexpanded EJS expression in
  `<script setup<%= useTs ? ' lang="ts"' : '' %>>`.
- Vux's loader `pre.vue` is a CoffeeScript/Pug input before its loader transforms.

Preparing these authored surfaces needs the appropriate actual upstream
transformation or reference parser and source mapping. It must retain every
original source and diagnostic, all 22 Matrix shards, and strict reference
usability. No ignore, waiver, fallback-to-empty finding set, or relaxed budget is
introduced here. Their owners and implementations remain unfinished under the
linked fixture/CI issues. This change does not claim full Matrix success or
release qualification.
