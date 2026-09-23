# Key manifests — the ambient inputs of every cached artifact

> [!NOTE]
> P5-1b. A cache key covering less than its artifact's inputs is a
> corruption bug, not a performance detail. Content is covered by the P5-1a
> artifact key; every other input is **ambient** and is declared here, per
> cached artifact. `vize_davinci::key::manifest` is the executable form of
> these two tables and `crates/vize_davinci/tests/key_manifests.rs` reads this
> file to prove the two agree row for row.

## Rules

1. Every cached artifact has exactly one row below, naming the stages its
   content key may come from and every ambient input it reads.
2. A `KeyManifest` folds into a key (`ArtifactKey::with_manifest`, or
   `KeyManifest::fingerprint` for an artifact with no content key) only when
   it sets **exactly** the declared inputs. A missing input is the corruption
   bug; an extra one is an input nobody declared. Both are errors.
3. Adding a cache, or a new input to an existing one, adds or edits its row
   here and in `CachedArtifact::inputs` in the same change; the test fails
   on either half alone.
4. Flipping any declared input changes the key; nothing else does — the
   manifest's insertion order and re-setting an unchanged value keep it.

## Ambient inputs

<!-- ambient-inputs:start -->

| Input               | What it covers                                                         |
| ------------------- | ---------------------------------------------------------------------- |
| `project-identity`  | the project's canonical root / `tsconfig.json` path                    |
| `tsconfig-content`  | the resolved `tsconfig.json` content, `extends` chain included         |
| `project-config`    | the Vize configuration a stage reads (Vue line, dialect options)       |
| `toolchain-version` | the Vize toolchain version that produced the artifact                  |
| `corsa-version`     | the Corsa (TypeScript) build the session runs                          |
| `feature-flags`     | every feature flag that changes the stage's output                     |
| `platform`          | the host target triple (a native session is not portable across hosts) |
| `plugin-identity`   | the JS plugin name and the filename passed in its batch                |
| `plugin-version`    | the JS plugin's declared version                                       |
| `plugin-code`       | the SDK digest of the JS plugin's rule sources                         |
| `plugin-visits`     | the node kinds the JS plugin requests                                  |
| `plugin-demands`    | the fact groups the JS plugin requests                                 |

<!-- ambient-inputs:end -->

## Cached artifacts

<!-- key-manifests:start -->

| Artifact                | Content key stages | Ambient inputs                                                                                                              | Consumer                |
| ----------------------- | ------------------ | --------------------------------------------------------------------------------------------------------------------------- | ----------------------- |
| `s0.source-block`       | `s0`               | `toolchain-version`                                                                                                         | resident tier (P5-4a)   |
| `s1.surface-page`       | `s1`               | `toolchain-version`, `feature-flags`                                                                                        | resident tier (P5-4a)   |
| `s2.page`               | `s2`               | `project-config`, `toolchain-version`, `feature-flags`                                                                      | resident tier (P5-4a)   |
| `projection.virtual-ts` | `s0`, `s2`         | `tsconfig-content`, `project-config`, `toolchain-version`, `feature-flags`                                                  | projection reuse (P5-7) |
| `corsa.session`         | none               | `project-identity`, `tsconfig-content`, `toolchain-version`, `corsa-version`, `feature-flags`, `platform`                   | Corsa sessions (P5-8)   |
| `plugin.result`         | `s0`               | `toolchain-version`, `feature-flags`, `plugin-identity`, `plugin-version`, `plugin-code`, `plugin-visits`, `plugin-demands` | JS plugin host (P5-13)  |

<!-- key-manifests:end -->

## Why each row reads what it reads

- **`s0.source-block`** — the SFC split is a pure function of the file bytes
  and the splitter; only the toolchain can change it.
- **`s1.surface-page`** — the lossless parse depends on the parser (toolchain)
  and its switches (`SurfaceParseOptions`, a feature flag). It is
  platform-independent: TS-43 checks equal keys on Linux and macOS.
- **`s2.page`** — the lowering also reads the project's Vue line
  (`LegacyCaps::for_version`), which is Vize configuration.
- **`projection.virtual-ts`** — the virtual TypeScript a block projects to
  depends on the tsconfig (module and JSX settings) and the Vize config on top
  of the block's own S0 or S2 key.
- **`corsa.session`** — a reused `ProjectSession` must never serve another
  project, tsconfig, Corsa build, flag set or host. `CorsaSessionKey`
  (`crates/vize_canon/src/corsa_session_cache.rs`, P5-8's first step) is this
  row's fingerprint: `SessionInputs` resolves all six inputs — the canonical
  tsconfig path, every config its `extends` chain reaches (path and bytes; an
  unresolved specifier by name), the Vize version, the Corsa build's identity
  (canonical path, size, mtime), the caller's flags and the host — and a test
  flips each one and asserts a different key. The session map and its
  lifecycle in `vize check-server` consume it.
- **`plugin.result`** — the full SFC source is the conservative S0 content
  key because diagnostic positions depend on text before the template. The
  batch also reads the filename, plugin code/version, visit list and demanded
  facts. These inputs are folded through the manifest; the toolchain version
  and compiled feature flags invalidate changes to S2 lowering or fact
  producers. The in-process and on-disk caches share this key.
