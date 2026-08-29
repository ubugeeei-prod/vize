# Source Registry Behavior

The source registry is the read-only manifest surface for future source-owned
install workflows tracked by issue #4896. It projects the existing
`uiFamilyCatalog` into deterministic JSON so tooling can discover the source
files, behavior contract, tests, type tests, dependencies, and bundle evidence
for each UI family without reading package exports.

| Command                | Output                                                     | Contract                                                                                                               |
| ---------------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `list`                 | JSON or JSONL family summaries in canonical catalog order. | Does not inspect user projects, write files, or resolve remote packages.                                               |
| `search <query>`       | JSON or JSONL matches with the fields that matched.        | Searches names, titles, aliases, upstream coverage, source paths, and quality gates with deterministic token matching. |
| `info <name-or-alias>` | JSON or JSONL full family manifest.                        | Resolves canonical names, package subpaths, titles, and exact aliases before returning source-owned artifacts.         |

## Non-goals

- It does not implement `init`, `add`, `add-many`, `remove`, `diff`, `update`,
  `doctor`, or `audit`.
- It does not create an offline cache, signed index, transaction journal,
  rollback path, or three-way update.
- It does not expose a new public package subpath.

Future mutating commands must keep dry-run and machine-readable output as a
first-class contract before writing user files.
