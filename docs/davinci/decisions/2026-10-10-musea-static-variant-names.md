# Musea static preview variant names

Issue: [#8470](https://github.com/ubugeeei-prod/vize/issues/8470).
Paired [decision](https://github.com/ubugeeei-prod/vize/issues/8470#issuecomment-6095606977).

The public npm 0.440.0 Musea build fails for the authored variant `State: enabled`.
The static runtime emitted the raw name after an art-path colon, while the loader
treated the final colon as the art/name delimiter. It therefore looked up
`Host.art.vue:State` and returned no module. Vite reported an unloadable dependency.

The original macOS diagnostic used unmodified installed public packages, the
actual native provider, and genuine `vite.build()` with Vize and Musea. Changing
only the variant to `State enabled` built 114 files; both ordinary previews
rendered the native button and scoped CSS through plain HTTP in Chromium.
The original consumer's 2,061 files/symlinks, custody and module hashes remained
unchanged. This is public macOS diagnostic evidence, not official Linux release
acceptance or evidence about a current source build.

## Decision

Use one `previewModuleId` producer in the static runtime and development preview
middleware. Escape percent signs first and colons second, then recover the name
once with the existing safe decoder. This preserves ordinary module IDs, literal
`%3A`, raw percent signs and authored names. Art paths and their drive/path colons
are unchanged. Existing preview filename hashes continue to use the authored
name; this change does not migrate filenames or snapshot identities.

Unit laws join the actual producer, resolver and loader for ordinary, colon,
percent, malformed percent, Unicode, quotes and Windows path inputs. Development
middleware uses that same producer. Exact fixture hashes retain the original red
and ordinary control; compiler collectors and differential expected outputs are
unchanged. The byte-exact product regression corpus lives in
`tests/tooling/fixtures/musea/static-variant-name`, outside the compiler corpus.
Adding these inputs under `tests/_fixtures` changed its fixed file vector from
498 to 502 despite zero remark changes; relocation restores the original 498
paths without changing collector rules, first-500 selection, goldens or budgets.
The retained `compiler-corpus-boundary.json` authenticates that exact pre-addition
main path vector and unchanged remarks baseline by SHA-256. The existing
formatter-sensitive custody policy covers this exact original-fixture directory;
formatting these retained inputs would erase their authenticated authored bytes.

## Genuine built HTTP contract

The source-native Actions job builds its exact native binding, Vize and the actual
Musea gallery. The new browser law calls production `vite.build()` with the real
plugins and unchanged literal example components/setup, plus the original
standalone colon fixture. It refuses the lightweight gallery fallback.

A plain HTTP server serves the emitted files under `/built/gallery/`, including
an ordinary static SPA fallback for document deep links. There is no Vite dev
server, intercepted preview module, substituted compiler or test runtime.
Chromium checks all six literal Button Self variants, native scoped CSS, disabled
state, globals at async setup readiness, three toolbar updates without replacing
the actual iframe Documents, retained query/hash and deep-link reload. Both
standalone native Host previews, including `State: enabled`, must render their
real button, CSS and setup globals. Response bytes/hashes, complete frame HTML on
failure, inputs, manifest, screenshots and build errors are retained as artifacts.

The earlier native dev law and original public acceptance helpers remain intact.
Source-native success does not prove installed public release acceptance.

## Delivery boundary

Preparation starts from the frozen native Self source head. Do not change that
PR, its observer child or any other active Stack. Open an independent main-base
PR only after native Self and its observer actually merge, incorporating their
real production clauses once. Fresh exact-head Actions, protected merge and
actual merge remain required. Release this hosted fix in the next minor only
after qualification; an earlier native/globals release may ship independently.
