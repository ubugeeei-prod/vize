---
title: Source Distribution (vize lib)
---

# Source Distribution (`vize lib`)

`vize lib` copies components from `@vizejs/ui` and composables from `@vizejs/composable` into your
project **as source**, in the style of shadcn/ui. You own the copied files: edit them freely, and
Vize keeps track of which package version each file came from so upgrades stay safe.

```bash
vpx vize lib pull rating
```

That writes the `rating` family plus everything it imports (`controllable-state`, `id`, ...) under
`src/components/vize/`, and records the pull in `vize-lib.lock.json`.

## Where the sources come from

Every published `@vizejs/ui` and `@vizejs/composable` tarball contains a versioned registry:

```text
node_modules/@vizejs/ui/
  registry/
    registry.json          # items, files, sha256 digests, dependencies
    files/families/...     # raw .vue / .ts / .css sources (tests excluded)
```

`vize lib` resolves a registry in this order:

1. `--registry <path>`: a `registry.json`, its directory, or an unpacked package directory. Repeat
   the flag to pass both the ui and the composable registry. Auto-discovery is then disabled.
2. The package installed in the project (`node_modules/@vizejs/ui/registry/registry.json`, looked up
   through parent directories like Node resolution).
3. `npm pack @vizejs/<pkg>@<version>` into a temporary directory followed by `tar -xzf`, when the
   request pins a version the installed package does not match, or when the package is not
   installed (then `@latest`). Pass `--offline` to forbid this step.

No registry server and no extra HTTP client are involved: a registry is exactly the tarball npm
already serves, so every version stays immutable and reproducible.

## Commands

| Command                                  | What it does                                                                   |
| ---------------------------------------- | ------------------------------------------------------------------------------ |
| `vize lib init [--dry-run]`              | Detect the project layout and write the `lib` config section.                  |
| `vize lib list [--kind ui\|composable]`  | List pullable items.                                                           |
| `vize lib search <words>`                | Match names, titles, descriptions, and aliases.                                |
| `vize lib info <name>`                   | Show files, registry dependencies, npm peers, and the package version.         |
| `vize lib pull <item>... [--dir <dir>]`  | Copy items and their registry dependencies; `--dry-run`, `--overwrite`.        |
| `vize lib add <item>...`                 | shadcn-compatible alias of `pull` (`-p/--path`, `-o/--overwrite`, `-y/--yes`). |
| `vize lib status`                        | Compare pulled files with the lockfile and the installed registry.             |
| `vize lib diff <name> [--to <version>]`  | Unified diff from your local copy to a registry version.                       |
| `vize lib update [<name>...] [--to <v>]` | Apply upstream changes without clobbering local edits; `--dry-run`, `--force`. |
| `vize lib remove <name>...`              | Delete items and dependencies nothing else needs; `--dry-run`, `--force`.      |
| `vize lib outdated`                      | Compare locked versions with the installed and latest registries.              |

Every command accepts `--json` for machine-readable output and `--root <dir>` to run against another
project.

### Naming items

Items are addressed by canonical name (`rating`), by alias (`star rating`, `useToggle`), with a kind
prefix when a name exists in both packages (`ui:locale`, `composable:locale`), and with an exact
package version (`rating@0.427.0`, `composable:use-toggle@0.427.0`).

### Target directories

Pulled files keep the registry layout (`families/form/rating/rating.vue`,
`foundations/id/deterministic-id.ts`, ...) below one directory per kind, so the relative imports
between items keep working without any rewriting. The directory is chosen by:

1. `--dir <dir>` (must stay inside the project),
2. the `lib` section of `vize.config.*`,
3. the registry default: `src/components/vize` (ui) and `src/composables/vize` (composable).

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  lib: {
    uiDir: "src/ui/vendor",
    composableDir: "src/composables/vendor",
    // dir: "src/vendor",        // shared fallback for both kinds
    // lockfile: "vize-lib.lock.json",
  },
});
```

Once a kind has been pulled into a directory, later pulls of that kind reuse it; a conflicting
`--dir` is rejected instead of splitting the dependency graph.

Pulled sources import only relative paths and npm packages such as `vue`. `pull` reports any npm
dependency your `package.json` does not declare yet; it never installs packages for you.

## Getting started: `init`

```bash
vize lib init --dry-run   # show the detected layout and the config change
vize lib init             # write it
```

`init` detects the source directory (`src/`, or `app/` for Nuxt 4 projects) and TypeScript, then
writes `lib.uiDir` / `lib.composableDir`. It creates `vize.config.json` when there is no config,
appends a `lib` section to an existing `vize.config.json` without touching the rest of the file,
and prints a snippet for `vize.config.ts` / `.pkl` instead of editing code. An existing `lib`
section is kept unless `--force` is passed. When `tsconfig.json` lacks
`allowImportingTsExtensions`, `init` says so: pulled sources import siblings as `./x.ts`.

## Checking for updates: `outdated`

`vize lib outdated` lists every pulled item whose registry differs from the lockfile:

| Column    | Meaning                                                                               |
| --------- | ------------------------------------------------------------------------------------- |
| `current` | Version recorded in `vize-lib.lock.json`.                                             |
| `wanted`  | Version of the registry `update` would use (installed package, else latest).          |
| `latest`  | Latest published npm version (`npm view`; skipped with `--offline`).                  |
| `state`   | `update-available`, `newer-release`, `removed-upstream`, `unknown` (or `up-to-date`). |

`update-available` means the item's `contentHash` differs; a version bump that does not touch an
item's files keeps it `up-to-date`. `--json` includes every item.

## Third-party registries

Any package or site can publish a registry in the same format and expose it under a namespace:

```json
{
  "lib": {
    "registries": {
      "@acme": "npm:@acme/vue-kit",
      "@design": { "source": "https://design.example.com/r/registry.json", "dir": "src/design" },
      "@local": { "source": "./registry", "dir": "src/local" }
    }
  }
}
```

```bash
vize lib pull @acme/data-table @design/button@2.1.0
vize lib list --kind @acme
```

- `npm:<package>[@range]` uses the installed package's `registry/registry.json`, else `npm pack`.
- `https://…/registry.json` is fetched with `curl`, and each file is downloaded from `files/<path>`
  next to it on demand (https only).
- Anything else is a path relative to the config file: a `registry.json`, its directory, or a
  package directory.

Third-party registries are validated against the same JSON Schema as the first-party ones (unknown
fields, malformed digests, unknown roles or dependencies, and an incomplete dependency closure are
rejected), and every downloaded byte is verified against its SHA-256 before it is written. A
namespace's items land in its `dir` (else the registry's `defaultTargetDirectory`), are locked under
the `@namespace` key, and can never overwrite a file that another pulled item owns.

## Versioning and safe updates

`vize-lib.lock.json` (commit it) records, per item, the package and exact version it came from, the
registry `contentHash`, whether you requested it or it came in as a dependency, and the SHA-256 of
every file as pulled. Those digests are the merge base for a three-way comparison between **your
file**, **the file as pulled**, and **the incoming registry file**:

| Your file vs. pulled | Registry vs. pulled | `update` / `pull` does                                |
| -------------------- | ------------------- | ----------------------------------------------------- |
| unchanged            | unchanged           | nothing (`unchanged`)                                 |
| unchanged            | changed             | replaces it (`update`)                                |
| edited               | unchanged           | keeps your edit (`keep-local`)                        |
| edited               | changed             | refuses (`conflict`) unless `--force` / `--overwrite` |
| unchanged            | removed upstream    | deletes it (`delete`)                                 |
| edited               | removed upstream    | refuses (`conflict-delete`) unless `--force`          |
| missing              | any                 | restores it (`create`)                                |
| exists, not locked   | any                 | refuses (`conflict`) unless `--overwrite`             |

Nothing is written while any conflict is unresolved, so a refused update leaves both the files and
the lockfile untouched. Use `vize lib diff <name> --to <version>` to review the upstream change, merge
it by hand, then run `update --force`.

A typical upgrade:

```bash
pnpm add @vizejs/ui@latest       # or: vize lib update --to 0.428.0
vize lib status                  # which items have updates / local edits
vize lib update --dry-run        # preview file actions
vize lib update                  # apply; conflicts are listed if any
```

`remove` deletes an item and every dependency pulled only for it. It refuses while another pulled
item still imports the target, and keeps locally edited files unless `--force` is given.

## Registry format

The registry document is described by
[`vize-lib-registry.schema.json`](https://github.com/ubugeeei-prod/vize/blob/main/npm/cli/schemas/vize-lib-registry.schema.json)
and the lockfile by
[`vize-lib-lock.schema.json`](https://github.com/ubugeeei-prod/vize/blob/main/npm/cli/schemas/vize-lib-lock.schema.json).
Both ship in the `vize` npm package under `schemas/`.

- `registryDependencies` is the complete transitive closure computed from the relative-import graph
  at build time; shared foundations are separate items, so pulling two components that both need
  `id` copies it once.
- Each file carries its `sha256`; `vize lib` verifies every byte it copies against it.
- `contentHash` changes only when an item's files change, so `status` reports "update available"
  only for items whose sources actually differ between versions.
