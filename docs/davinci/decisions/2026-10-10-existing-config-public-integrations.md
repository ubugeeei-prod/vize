# Public integration configuration inventory (#8371)

The inventory below covers the remaining public integrations outside the core
CLI, compiler Vite plugin, Vite+ tasks, and editor configuration loaders. Those
core entry points share configuration discovery and projection implemented for
[#8371](https://github.com/ubugeeei-prod/vize/issues/8371).

| Surface                                                             | Configuration entry point                                                                                                      | Dedicated configuration requirement                                                                              |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------- |
| Musea gallery, `musea()`                                            | Plugin options in `vite.config.*`; top-level `vize.musea`; exact shared Vize plugin registration when present                  | None. A standalone Musea plugin projects the Vite configuration already evaluated by Vite.                       |
| `musea-vrt`                                                         | Selected Vite config (`--config` supported), plugin options, top-level `vize.musea`, defaults                                  | None. The private dedicated-file loader is replaced by the shared loader.                                        |
| `vize musea new`                                                    | Default art-file discovery; optional `vize.musea` settings in Vite                                                             | None. Generates `stories/Button.art.vue` without creating or altering configuration files.                       |
| `vize musea` / `vize musea --build`                                 | Existing Vite installation plus `musea()` plugin in Vite config                                                                | None. Nuxt users continue to use their Nuxt development/build commands.                                          |
| `vize lib init`                                                     | Existing Vite `vize.lib`; fresh `vite.config.mjs`; detected source layout                                                      | None. Fresh setup creates `vite.config.mjs`, while existing code configurations are preserved.                   |
| `vize lib list/search/info/pull/status/diff/update/remove/outdated` | Shared `lib` settings, public command flags, registry defaults                                                                 | None. Source CLI regression proves `init` followed by an offline `pull` honors the new Vite configuration.       |
| Nuxt lint config generation                                         | Shared config's `linter` settings plus Nuxt module options                                                                     | None. Project rules use the canonical loader with ancestor discovery rather than requiring `vize.config.json`.   |
| Nuxt compiler and Musea integration                                 | Nuxt's existing `vize` module options forwarded to builder plugins                                                             | None. Nuxt remains a framework-owned configuration entry point.                                                  |
| Musea MCP server                                                    | `createMuseaServer` options; `musea-mcp` project-root argument/environment; token-path argument/environment; art-file defaults | None. MCP does not currently discover shared Vite settings. Its public factory accepts include/exclude directly. |
| Devtools                                                            | `createDevtoolsTraceRecorder`, manifest and runtime API options                                                                | None. No project-config IO.                                                                                      |
| unplugin (Rollup, Webpack, Rolldown, esbuild, Babel)                | Explicit plugin/compiler options in the host tool config plus defaults                                                         | None. No dedicated or shared project-config IO.                                                                  |
| Rspack plugin                                                       | Host Rspack plugin options plus defaults                                                                                       | None. No dedicated project-config IO.                                                                            |
| Fresco, compose, Marquette                                          | Their public APIs and host/runtime options                                                                                     | None. No dedicated project-config IO.                                                                            |
| oxlint plugin                                                       | Existing oxlint config or Vite+ lint task configuration                                                                        | None. Scoped lint config is the host tool's configuration.                                                       |

## Precedence and evaluation

For an already evaluated Vite configuration, the precedence is explicit Musea
plugin options, a dedicated Vize config in the configuration's own directory,
then that Vite configuration's top-level `vize` settings, then defaults. VRT
options and include/exclude are resolved per option, so a plugin's explicit
include does not discard a shared exclude. Shared `musea.vrt.outDir` maps to the
VRT runner's snapshot directory.

The Musea CLI projects the exact Vite configuration selected by its `--config`
argument, including custom filenames and dynamic Vite config exports. It does
not re-import that file through the shared loader. The plugin follows the same
rule in `configResolved`. This prevents repeated user side effects and recursive
plugin setup. If no Vite configuration is selected, ordinary canonical ancestor
discovery applies. Nuxt lint rules use canonical ancestor discovery from the
Nuxt project root.

Existing dedicated JSON library configurations keep their original append and
forced-replacement behavior. Existing code configurations remain untouched and
receive a snippet when a library section still needs to be added. Symlink and
project-boundary protections apply to the new Vite config destination too.

## Regression evidence and bounds

- `crates/vize/tests/config_init_cli.rs` invokes the public source-built CLI for
  Musea creation, fresh library setup, actual offline source pull, and existing
  Vite configuration preservation. Its registry has committed source bytes and
  digests in `crates/vize/tests/fixtures/config-init/`.
- Library command tests preserve existing dedicated JSON and code configurations,
  dry-run behavior, malformed config refusal, and symlink refusal.
- Musea CLI tests exercise top-level Vite settings, per-option plugin overrides,
  dedicated JSON precedence, VRT options, custom Vite filenames, and one config
  evaluation.
- Musea plugin tests cover exact Vite registration, pending registration, plugin
  ordering, and standalone projection without a compiler plugin registration.
- Nuxt tests exercise shared Vite lint rules, preset expansion, nearest monorepo
  config discovery, and dedicated-config precedence.

These regression tests verify configuration behavior. They do not claim to
complete Musea's separate hosted test runner, VRT quality, props editor, or
remaining Devtools/unplugin experimental product work. Those surfaces already
accept options without a dedicated configuration file and keep their current
support boundaries.
