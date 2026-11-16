# Integrated plugin API v2 incubation

> [!WARNING]
> Plugin API v2 is experimental. This page records what is implemented and the
> decisions the [API draft](./integrated-plugin-api-draft.md) leaves open. It
> does not change the v1 scope. Progress is tracked in
> [#13074](https://github.com/ubugeeei-prod/vize/issues/13074).

## Status

| Slice                                                    | State                                    |
| -------------------------------------------------------- | ---------------------------------------- |
| API v1 packet freeze                                     | done (#13122)                            |
| v2 manifest, canonical plan, validation, inspector views | done (#13124)                            |
| `vue.plugins` resolved from `vite.config.*`              | done                                     |
| custom lint (`lint.rule`)                                | planned                                  |
| custom directive (`vue.directive`)                       | declared only; every consumer refuses it |
| custom block (`sfc.custom-block`)                        | declared only; every consumer refuses it |

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
  not admit. No product admits a v2 capability until its slice lands.
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
- **Global refusal.** Until a capability is implemented by some product, every
  product refuses a plan that contains it at config load. `vize plugin inspect`
  still reports the plan with its refused capabilities.
- **Composition.** A later `withVue({ plugins })` replaces the list, like other
  Vue option arrays.
- **Serialized configs.** A serialized Vite config (`--config-stdin`) may carry
  only the resolved `__vizePluginPlan`. Raw entries lost their callbacks, so they
  are refused.
- **Directives.** Names are kebab-case and cannot shadow a built-in directive.
  Modifiers normalize to `{ optional: true, default: false }`. A required
  modifier with a default is refused.
