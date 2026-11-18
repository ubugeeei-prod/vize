# Integrated plugin API v2 incubation

> [!WARNING]
> Plugin API v2 is experimental. This page records what is implemented and the
> decisions the [API draft](./integrated-plugin-api-draft.md) leaves open. It
> does not change the v1 scope. Progress is tracked in
> [#13074](https://github.com/ubugeeei-prod/vize/issues/13074).

## Status

| Slice                                                    | State                                                 |
| -------------------------------------------------------- | ----------------------------------------------------- |
| API v1 packet freeze                                     | done (#13122)                                         |
| v2 manifest, canonical plan, validation, inspector views | done (#13124)                                         |
| `vue.plugins` resolved from `vite.config.*`              | done                                                  |
| custom lint (`lint.rule`, adapted v1 rules)              | done: `vize lint` incl. `--fix`, LSP, Oxlint and Nuxt |
| custom directive (`vue.directive`)                       | lint, LSP and compiler; types pending                 |
| custom block (`sfc.custom-block`)                        | declared only; every consumer refuses it              |

## Where the contract lives

- Authoring: `@vizejs/plugin-sdk/v2` (`npm/plugin-sdk/v2.js`). `defineVizePlugin`
  returns a factory. Each call runs `setup` once and returns a frozen entry. The
  entry holds the lowered plain-data manifest and its callbacks, keyed by the
  registered symbol `Symbol.for("vize.plugin.entry")`.
- Resolution: `crates/vize_carton/src/config/loader/plugin-plan.js`, beside the
  shared `vite.config.*` projector. This is the only plan builder.
- Native verification: `vize_carton::config::plugin_plan`. It re-serializes the
  plan canonically and checks the digest, registry versions, stable order and
  claims. It never rebuilds the plan.
- Registry: `crates/vize_carton/src/config/loader/plugin-registry.json`, read by
  both sides. It lists the plan schema, API versions, products, backends, and
  each capability's schema versions, consumers and admitting products.
- Shared fixtures: `tests/_fixtures/plugin-plan/resolve` (JS resolves, Rust
  verifies the same digest) and `tests/_fixtures/plugin-plan/verify` (tampered
  plans that Rust must refuse).
- v1 freeze: `tests/tooling/davinci-plugin-api-v1-freeze.test.ts` with
  `tests/tooling/fixtures/davinci-plugin-sdk/api-v1-packets.json`.

- Configuration: the shared projector (`loader/vite-runtime.mjs`) resolves
  `vue.plugins` once for Vite, Vite+, Nuxt, the CLI and the LSP. A non-empty plan
  travels as the host-only `__vizePluginPlan: { digest, plan }` key. In-process
  JS hosts read the resolved plan, with its callbacks, under
  `Symbol.for("vize.plugin.plan")`. Every native document parse verifies the
  plan. `vize plugin inspect [--config <path>]` prints the plan and per-product
  admission.

Every refusal is a `[plugin/<code>] <subject>: <message>` error. JS and Rust
use the same codes.

## Decisions

Each choice below is the conservative option. It can be relaxed later without
invalidating a plan that is valid today.

- **v1 freeze scope.** The v1 packet freeze pins the v1 protocol: packet schema,
  fields and framing. Changing them needs a versioned v1 break. A correction of
  the template tree a plugin sees, which aligns the parse with Vue, may
  regenerate the golden, recorded here with the API version unchanged:
  - #13307: an SFC template keeps its authored table rows, as Vue does. So
    `<table><tr>` no longer yields an implied `tbody` node. In the packets of
    `lint/team-conventions`, `lint/two-plugins-audited` and `lint/fact-provider`,
    which read `TODO_LIST`, every later node id and parent index moves down by
    one.
- **Canonical data.** Plans are compact JSON with object keys in UTF-8 byte order.
  Numbers must be safe integers; other numbers are refused, so JS and Rust need no
  floating-point formatting agreement. Lone surrogates, accessors, symbols,
  cycles and non-plain objects are refused. Object members whose value is
  `undefined` are omitted, as in JSON.
- **Entries.** `false`, `null` and `undefined` entries are skipped like
  conditional Vite plugins. Nested arrays and promises are refused. A v1
  `definePlugin` rule plugin is adapted as one `lint.plugin-v1` capability, and
  the original object is executed unchanged. Other v1 hooks keep their native
  APIs.
- **Identity.** Plugin names are package-style and unique within a plan. Rule
  IDs are `<plugin>/<rule>`. The SDK fingerprint covers the SDK runtime, the
  lowered manifest, `setup` and every callback source.
- **Ordering.** `before`/`after` must name plugins present in the plan. Among
  ready plugins the authored order wins. There is no numeric priority.
- **Ownership.** Claim keys are exclusive. A plugin that claims a key twice also
  conflicts. A custom block claimed for every language conflicts with the same
  block claimed for one language.
- **Capabilities.** An unknown or unsupported capability marked
  `required: false` is skipped and listed under `skipped`. A required one fails.
  A product refuses a plan containing a capability addressed to it that it does
  not admit. A product admits a capability once it implements its part.
- **Backends.** Only `js` is admitted. `native` and `wasm` fail with
  `backend-unavailable`, and there is no `auto`. Nothing falls back silently.
- **Runtimes.** The resolving runtime (`node` or `bun`) must appear in
  `supports.runtimes`.
- **Permissions.** `env` keys and `fs.read` paths are declared and inspectable.
  They do not sandbox trusted JavaScript.
- **Empty plans.** An absent list, `[]` and a list of only skipped entries all
  project byte-identically to a config without plugins: no key, no file, no
  callback. The plugin host (`loader/plugin-host.js`) is imported only when a
  list has an entry. The native config runtime carries it as a lazily imported
  `data:` module, so configs without plugins never parse or evaluate it.
  Inspection and other plugin tools (`loader/plugin-tools.js`) are only in the
  separate tooling runtime.
- **Global refusal.** Until every product that consumes a capability
  implements it, every product refuses a plan that contains it at config load,
  so no product silently ignores what another one honors. `vize plugin inspect`
  still reports the plan with its refused capabilities.
- **Composition.** A later `withVue({ plugins })` replaces the list, like other
  Vue option arrays.
- **Serialized configs.** A serialized Vite config (`--config-stdin`) may carry
  only the resolved `__vizePluginPlan`. Raw entries lost their callbacks, so they
  are refused.
- **Directives.** Names are kebab-case and cannot shadow a built-in directive.
  Modifiers normalize to `{ optional: true, default: false }`. A required
  modifier with a default is refused. `VerifiedPluginPlan::directives` lowers
  every contract once into the native table that each consumer will read.
  `normalized_modifiers` keeps a binding's authored modifiers in order and
  appends every absent modifier that defaults to `true`, in canonical order. It
  refuses unknown modifiers and missing required ones.
  `vize lint`, the language server and the Oxlint integration check every
  binding against its contract natively, without a sidecar, and report a
  violation on the binding's authored span as `plugin/directive-contract`.
  The compiler admits the capability too. A declared directive's absent
  modifiers that default to `true` are appended after the authored ones in
  VDOM, SSR and Vapor output, by the L2 lowering and by the legacy transform
  alike, so every lane emits the same modifiers object. A VDOM template that
  may bind a declared directive takes the compatibility lane, because the
  published `DomEmitOptions` literal cannot carry the table. A binding that breaks
  its contract fails the template with `compiler/directive-contract` on its
  authored span. Hosts admit the plan as the compiler once. The
  table carries each declaring plugin's `supports.outputs`, and the SFC
  compiler, which resolves each file's output (SSR, a `vapor` script or
  request, otherwise VDOM), refuses a file compiled to an output its plugin
  lacks (`host-unsupported`).
  The plan still loads nowhere until the type checker implements the
  capability too. The Vite host registers the resolved plan with the native
  compiler once (`registerPluginPlan`), which re-verifies it and lowers the
  table; every compile then names the plan by its digest and refuses an
  unknown digest (`plan-stale`). Registered plans stay for the process, so
  a host's digest never expires while its config is unchanged. JSX/TSX
  templates do not read the table yet.

## Plugin lint rules

`vize lint` (and the Vite+ lint task, which runs it) executes admitted
`lint.rule` capabilities and v1 `definePlugin` rule plugins listed in
`vue.plugins`.

- **Records and facts.** Schema 1 rules visit the same public L2 template
  records and demand the same native fact groups as plugin API v1. The host
  builds them with `vize_patina::plugin_batch`, so an adapted v1 plugin receives
  byte-identical batches.
- **Severity.** The root `vue.linter.rules` sets each plugin rule ID
  (`<plugin>/<rule>`) to `off`, `warn` or `error`. Otherwise a v2 rule uses
  `meta.defaultLevel` (default `off`) and a v1 rule warns. Plugin rules cannot
  be configured in `entries[]`. Rules that are off add no work.
- **One crossing.** Every enabled job and every Vue document travel in one
  request to a trusted Node sidecar. The sidecar starts as soon as a plugin rule
  is enabled, so Node startup and its own `vite.config.*` evaluation overlap the
  built-in pass. One process serves every pass of the run. It refuses a plan
  whose digest differs (`plan-stale`). Each rule runs once per document.
  Reports name visited records; the host owns spans, severity, message
  rendering (`{name}` placeholders from bounded `data`) and ordering.
- **Receipt.** `vize lint --profile` prints a `vize_plugin_receipt:` JSON
  line: crossings, processes, documents, request and response bytes, sidecar
  round-trip time, and per job the batches, records, batch bytes, reports and
  JavaScript time.
- **Fixes.** A fix replaces the reported record's authored span: the whole
  attribute for `ui.bind`, and the whole text run (adjacent text and every
  mustache) for `ui.interpolation`. `--fix` applies only the fixes of rules
  declared `fixable: "safe"`. It never applies unsafe fixes or v1 suggestions.
  Each pass fixes, re-lints (built-in rules and their fixes) and writes the
  changed files in parallel, one write per file. The plugins then see only
  those files, at one more crossing per pass, for at most five passes. A file
  whose plugin fix the built-in fixes revert is left unwritten.
- **Editor.** The language server runs the same jobs on every Vue document it
  lints. A config generation that enables plugin rules owns one sidecar,
  started on the first lint. Edits reuse it and never start a process. Each
  lint sends one document with every job. The sidecar must resolve its config
  within 30 s and answer within 2 s, or it is stopped and restarted on the
  next lint. A newer document version cancels the wait, and the late answer
  is skipped. A planning error, a rule failure or a timeout is one error at
  the start of the document (code `plugin`). After three consecutive sidecar
  failures the server reports the last one until the config changes. A rule
  failure is answered, so the sidecar keeps running.
- **Editor fixes.** A plugin diagnostic with a fix offers it as a quick fix,
  preferred only when its rule is `fixable: "safe"`. The editor keeps the last
  answer per document text, so diagnostics, hover and code actions of one text
  cross into JS once.
- **Oxlint and Nuxt.** oxlint-plugin-vize hosts the rules with
  `settings.vize.pluginRules: true`. It runs the plugins of the nearest
  `vite.config.*` above each Vue file. One native `PluginLintSession` per config
  owns one sidecar for the whole Oxlint process, and each Vue file crosses into
  it once. `vize/plugin-errors` reports the plugin diagnostics Vize resolved as
  errors and `vize/plugin-warnings` the rest, so severity stays the one set in
  `vue.linter.rules`. A session or rule failure is one error on the file. The
  Nuxt lint config enables both rules and the setting when `vue.plugins` lists
  entries. It keeps plugin rule IDs out of the Oxlint rule map, refuses them in
  `entries[]`, and refuses capabilities lint does not implement. Like every
  Oxlint rule on a `.vue` file, they run only for files with a `<script>`
  block.
- **Not yet.** Templates that the plugin records cannot represent are skipped
  (and listed in the CLI receipt).
