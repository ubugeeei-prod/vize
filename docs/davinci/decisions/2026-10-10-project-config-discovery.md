# Project configuration discovery

Issue: [#8371](https://github.com/ubugeeei-prod/vize/issues/8371).

Vite and TypeScript project files should suffice for the compiler, linter,
formatter, checker, editor, Musea and source-library workflows. Dedicated Vize
configuration remains compatible. Initializers preserve user files and do not
create a dedicated configuration.

## Shared configuration

An ordinary `vite.config.*` carries native settings in a top-level `vize`
section. Vite+ configurations additionally project `compiler`, `typecheck`,
`lint.vize`, and `fmt.vize`. Common `fmt.sortImports` also applies to Vize;
other native formatter options belong under `fmt.vize`, preserving the existing
Vite+ task payload and formatting history. Fresh native check tasks enable JSX
without adding unrelated checker defaults to formatter or lint payloads.
Native settings include scoped entries, globals, editor features, Musea and the
source library. TypeScript options and references remain in `tsconfig.json`.

The dependency-free projection lives in
`crates/vize_carton/src/config/loader/vite-runtime.mjs`. Native loaders embed it
from their publishable crate; the npm CLI bundles the same module. Published
consumers need no repository-relative files. The CLI adds no Vite runtime
dependency. Evaluation uses the existing Node configuration path. The Vite+
factory's source symbol permits reading settings without starting its plugins
or generated tasks.

## Precedence and roots

1. An explicit config file wins. `--no-config`, loader `mode: "none"`, and plugin
   `configMode: false` retain their opt-out behavior.
2. Within one directory, dedicated configs retain their previous order: Pkl,
   TypeScript, JavaScript, ESM, JSON. They precede Vite files.
3. Vite discovery uses TypeScript, JavaScript, ESM, CommonJS, MTS, CTS.
4. With no dedicated file, project defaults enable standalone JSX checking.
   Dedicated files retain their historical feature defaults.

Automatic CLI discovery ascends until the nearest package, TypeScript or
JavaScript project boundary. An unconfigured monorepo package does not inherit
a parent or sibling policy. Explicit roots stay shallow, matching LSP
workspace-folder isolation and npm `mode: "root"`. Relative native paths belong
to the selected config's directory; explicit CLI paths belong to the invocation.

Vite hooks use the already evaluated configuration with a dedicated-only disk
lookup, avoiding a second import of the active Vite file. Explicit plugin
options retain their precedence over shared settings. Vite+ retains explicit
tool-section overrides over inherited native settings.

Fresh editor projects expose formatting with the existing recommended editor
profile. Project `languageServer`/`lsp` switches override defaults; explicit
editor switches override project settings. Dedicated projects retain formatter
opt-in behavior. The experimental JSX capability also respects the effective
typecheck switch, so an explicit disabled editor profile advertises it as off.
Original disabled-profile protocol oracles remain unchanged.

## Qualification and remaining boundaries

Public CLI/LSP, packaged initializer and editor regressions qualify customized
settings, defaults, positive/negative controls, explicit overrides, dedicated
compatibility and monorepo package boundaries. Combined source Actions and
protected queue qualification are required before actual delivery.

Fresh JSX defaults additionally depend on retaining authored TSX slot-parameter
annotations in the native typecheck-only lowering path. The existing complete
public TSX contract oracle must pass unchanged; a configuration test cannot
qualify a native diagnostic false negative.

The LSP retains one process-wide capability/type-checker profile. Root scoped
entries and explicit workspace folders provide monorepo policies. Automatic
per-document adoption of separate nested Vite configs remains unfinished.
Vite `root` selects the bundler root; standalone source selection still follows
the invocation and explicit patterns/TypeScript project. Root parity remains
unfinished without a source-bound public regression. This change does not
close #8371's complete acceptance until these remaining boundaries are resolved.

The authored #8371 acceptance deliberately enables usable formatting in fresh
projects without dedicated configuration. The Zed regression's original complete
`case.json`, five dedicated/explicit vectors, source bytes and hashes remain
archival authority. `current-default-policy.json` declares only the no-config
formatting-capability successor, pins the historical manifest hash, and requires
the entire formatted buffer after applying every protocol edit. An additive
explicit `formatting: false` control retains absent-provider and null-result
behavior. No other historical expectation or public JSX contract is relaxed.
The Nix source filter must retain the canonical embedded `.mjs` runtime asset.

Invalid editor configuration is not a config-free fresh project. The shared LSP
snapshot records validity after the same single checked evaluation; malformed
dedicated/Vite files retain the historical editor fallback without applying new
fresh defaults. Original invalid-JSON whole feature expectations remain unchanged.
Explicit CLI missing files retain the historical `config file not found` message.

The first-layer formatting process fixture explicitly disables editor type checking. Its JSX routing capability therefore remains false even when the stored fresh JSX feature is true. Preserve both original formatting inputs and outputs; an additive active-typecheck process control advertises the native JSX route. The independent fresh-project whole diagnostic cases retain their original positive and TS2322 packets.

The same preserved formatting packet exposed that the editor omitted Vite compiler whitespace. Carry this value from the single checked evaluation with formatter options and use the existing native Glyph builder for document, range and on-type formatting. Vite `preserve` now matches the CLI; dedicated formatter behavior remains unchanged.

## Packed initializer plan successor

The original #3956 TypeScript, JavaScript/checkJs and Vite+ shape modules remain
unchanged as the named dedicated-config baseline. The release driver selects
`CONFIG_FREE_PROJECT_SHAPES` for the current #8371 initializer: omit only the
generated `vize.config.ts` and replace the complete feature/discovery plan with
the project-settings/defaults outcome. Retain every other complete generated
file, authored source, dependency plan, manager invocation and clean/broken/
repaired diagnostic vector. Both initialization passes must leave every
`vize.config.*` absent. Existing dedicated-user compatibility controls remain.
Pure projection controls do not qualify a packed installation; fresh Actions
and the actual generated-project runtime remain required before release credit.

The original release input audit retains all 37 historical contracts, 31 scoped
contracts and six complete broad selections. Its explicit current successor
adds only the config-free initializer contract to the broad set; unresolved
dynamic source reads remain broad. Actions must validate the generated formatter
inventory against the current source addresses; updating those addresses does
not change a provider, consumer, symbol or import count.

The actual #8516 delivery adds the existing Yarn JavaScript/checkJs cell and
strict completed-child outcomes. Preserve its whole authored input, manager
arguments, diagnostic triples and null-signal exit-zero/one controls. The
separate current Yarn packet changes only three complete initializer stdout
plans and the omitted dedicated config file, retaining the original archive.
The current release audit is exactly 40/31: all original 37 contracts, the two
genuine delivered broad contracts, and the named config-free broad contract.
Packed execution across all eight manager cells remains an Actions obligation.

The canonical Rust packed producer runs the installed TS/JS typecheck helper
before the eight fresh-project cells. Its current generated-file successor
snapshots only `package.json` and `tsconfig.json`; the same existing fresh-project
absence check rejects every `vize.config.*` after both initialization passes.
Preserve the helper API, authored TS/JS sources, diagnostic checks, manager argv
and idempotency flow. The entire original helper and runtime-oracle source remain
archived under `project-settings-8371/archives`. The false-clean oracle retains
its whole historical fake CLI literal and derives the named current fixture by
omitting only its generated dedicated-config write; its original standalone TS
failure assertion and invocation remain unchanged. No wrapper, parser or runtime
stage is added. Node fixture and inverse controls do not qualify actual packed
execution or a public release; fresh exact-source Actions remain required.

## Review and execution successors

Vite config functions receive only `serve` or `build`. Vize workflow commands
such as `lint`, `check` and `fmt` project to `serve`, preserving every other
supplied environment field and leaving the caller's object unchanged. Existing
valid Vite environments retain their object identity. Dedicated Vize config
functions continue receiving the authored workflow command.

The original sub-package checker runner is retained as a complete
[historical execution recipe](../../../tests/_fixtures/differential/typechecker/sub-package-3366/original-checker-runner.ts.txt).
Its current recipe adds only `--no-config`: the intentionally unresolvable
`vize.config.ts` remains an unchanged checked source, including both original
installed/missing-module diagnostic controls. Default configuration discovery
still refuses the same missing import before checking, with no silent fallback.
The original full Check failure and complete runner remain archival authority.

An additive `fresh_formatting_without_an_authored_switch_returns_the_complete_edit_packet`
process law omits only the authored formatting switch from a separate input.
It requires all three providers, the complete diagnostic publication, the whole
single-edit response including range and formatted buffer, graceful shutdown,
and absence of dedicated configuration. Preserve all original 32 test inputs,
oracles and raw observations, including 13 lifecycle cases and 16 authenticated
processes. The named current native qualification successor requires exactly
33 passing tests and 17 joined processes; the added law cannot borrow the
original 32/16 head's Actions credit or weaken any build/executable/protocol join.

The appended default-discovery missing-import case keeps its complete authored
input, exit-two requirement and full diagnostics when it moves from the tooling
runner to the checked native owner in #8476. Retain the entire old runner and
whole test block in source custody; the preceding six cases remain byte-exact.
This changes execution ownership only, with no product behavior or expected
outcome change. The additive no-authored-formatting law compares its entire
fixture with the original settings minus only `formatting: true, `, preserving
all original fixtures, editor response oracles and native process joins.
The complete old tooling, native, consumer and full-script failure logs remain
authority. Every changed layer needs fresh exact-source Actions; cached CLI
results and prior green heads do not qualify this successor.

## Core-owned Musea build prerequisite

Protected core candidate `55b31b19c6cfff9db87fb3ef4a6552edd6b4f7da` failed the unchanged native inline and packed-gallery contracts because the Vite plugin imports the new `vize/config` entry, while its job had not built the CLI package. The exact existing CLI-before-Vite build command belongs to this config-producing core slice and is moved down from the public integration layer. Native acquisition, all browser commands, assertions, artifacts, job budgets and deadlines remain intact. The failed candidate and its whole first-failure log remain historical; fresh core Actions and protected candidate qualification are required.

The owning prerequisite also includes the unchanged existing `setup-rust-script` action: CLI `build` runs workspace `generate:rule-types`, whose task calls `rust-script tools/commands/generate/rule-types.rs` before `vp pack`. The filtered Vite dependency closure includes CLI build dependencies, and the Rust generator reads the committed Patina rule snapshot through the existing common helper. No native test command or job budget changes.

The actual merged Musea service branch is composed with the existing single-evaluation configuration aggregate. The unchanged `serve` registration, session protocol, and complete error/return branch are retained; its VRT options come from that same loaded snapshot before dispatch, while preview and file scanning retain their existing command order. The complete pure worker-count parser moves to the existing CLI utility owner without changing its body so both source files remain within the existing 350-line limit.
