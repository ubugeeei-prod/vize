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
