# Validate qualification contracts at their dependency boundaries

Refs [#6830](https://github.com/ubugeeei-prod/vize/issues/6830),
[#9200](https://github.com/ubugeeei-prod/vize/pull/9200).

Full Check [38929265536](https://github.com/ubugeeei-prod/vize/actions/runs/38929265536)
at `a00fb6cff9268e1371ba260739d905050e2a2573` failed five tooling
assertions across four workers. Their contracts depended on YAML presentation,
documentation positions or undeclared dependency access. Repair those tests
without changing product code, the retained fixture bytes or the required
qualification jobs.

Read the playground job's parsed `needs` value, accepting equivalent YAML flow
and block sequences. Compare complete Japanese directive packets and their
English/source code blocks, restoring every section heading that actually
exists in the original. Check each retained composable example by its unique
public identity, independently of insertion order and total catalogue size.
For the authored `vue/valid-v-on` option row, verify its exact API tokens and
real link target separately; preserve the original translation's complete
recipe and inline-token digests. These checks remain scheduled/manual tooling
qualification rather than additional PR product gates.

The legacy instance API oracle has always authenticated Vue `3.6.0-rc.9`,
runtime-core source bytes and TypeScript `6.0.3`. Declare a tests-owned Vue alias
for that exact existing lock graph instead of borrowing the benchmark
workspace's different Vue version. Resolve runtime-core through Vue's actual
runtime-dom dependency, and TypeScript through the selected parser's peer
dependency. Confirm the authenticated runtime-core module was loaded. Explicit
external-manifest overrides remain supported, with the same version and hash
checks; missing dependencies or mismatched providers still fail.

Local controls passed for the documentation and workflow tests. The actual
legacy runtime replay passed both tests without skips; wrong-version,
modified-runtime and missing-alias controls failed. All 32 retained oracle
fixture files are unchanged. An isolated 35-importer installation passed
pnpm `12.1.0` frozen-lock and policy validation; the repository lock adds only
the three alias entries. A fresh combined-source Actions run remains required.

The separate native handoff signal repair is recorded in the
[parser/semantic handoff decision](./2026-10-16-canon-parser-semantic-handoff.md).
The VS Code completion failure and Inspector baseline decision remain open;
these tooling repairs do not establish complete main or release health.
