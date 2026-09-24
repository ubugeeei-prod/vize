# Source Registry Behavior

The source registry is the repository-only, read-only manifest surface for
future source-owned install workflows tracked by issue #4896. It projects the existing
`uiFamilyCatalog` into deterministic JSON so tooling can discover the source
files, behavior contract, tests, type tests, dependencies, and bundle evidence
for each UI family without reading package exports.

| Command                | Output                                                     | Contract                                                                                                               |
| ---------------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `list`                 | JSON or JSONL family summaries in canonical catalog order. | Does not inspect user projects, write files, or resolve remote packages.                                               |
| `search <query>`       | JSON or JSONL matches with the fields that matched.        | Searches names, titles, aliases, upstream coverage, source paths, and quality gates with deterministic token matching. |
| `info <name-or-alias>` | JSON or JSONL full family manifest.                        | Resolves canonical names, package subpaths, titles, and exact aliases before returning source-owned artifacts.         |

## Published registry

`pnpm build` also runs `scripts/build-source-registry.ts`, which writes
`registry/registry.json` plus the raw, non-test source files under
`registry/files/` into the package directory; `package.json#files` ships both
in the npm tarball. The shape is documented by
`npm/cli/schemas/vize-lib-registry.schema.json` and consumed by `vize lib`.

| Input                                                 | Outcome                                                                                                   | Proven by                                                              |
| ----------------------------------------------------- | --------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| Catalog family                                        | One registry item with the same name, title, aliases, entry, and package subpath.                         | `publishes one registry item per catalog family with package identity` |
| Two builds from the same checkout                     | Byte-identical `registry.json`.                                                                           | `is byte-for-byte deterministic`                                       |
| Built manifest                                        | Validates against the published JSON Schema.                                                              | `conforms to the published JSON Schema`                                |
| Catalogued source file                                | Published once, owned by one item, with its SHA-256 and size; test files never ship.                      | `hashes every published file and item content`                         |
| Relative import from an item file                     | Resolves to a file owned by the item or by an item in its `registryDependencies`.                         | `resolves every relative import inside the item's registry closure`    |
| Dependency of a dependency                            | Listed in the importing item's `registryDependencies` (transitive closure).                               | `registry dependencies are transitively closed and acyclic in naming`  |
| Shared foundation or uncatalogued support module      | Foundations stay separate items; support modules are claimed by the first item (name order) importing it. | `pulls shared foundations as their own items`                          |
| Bare import                                           | Recorded as an npm dependency with the package's declared range and `peer` / `runtime` kind.              | `declares vue as the only npm dependency, as a peer`                   |
| Relative import that escapes `src/` or a missing file | The build throws; no registry is written.                                                                 | `rejects unresolvable relative imports and undeclared packages`        |
| Rebuild over an existing `registry/`                  | The directory is replaced, so removed files cannot linger.                                                | `writes registry.json and raw files, replacing stale output`           |

## Non-goals

- It does not implement `init`, `add`, `add-many`, `remove`, `diff`, `update`,
  `doctor`, or `audit`.
- It does not create an offline cache, signed index, transaction journal,
  rollback path, or three-way update.
- It does not expose a new public package subpath.
- It does not publish a package bin or support execution from an installed
  `@vizejs/ui` consumer project; installed projects use the published
  `registry/registry.json` through the `vize lib` CLI instead.

Future mutating commands must keep dry-run and machine-readable output as a
first-class contract before writing user files.
