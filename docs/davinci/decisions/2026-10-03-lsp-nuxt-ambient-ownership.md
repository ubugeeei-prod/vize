# LSP Nuxt ambient declaration ownership (2026-10-03)

Tracking: [#3952](https://github.com/ubugeeei-prod/vize/issues/3952) and
[#6883](https://github.com/ubugeeei-prod/vize/issues/6883).

The original Real Project Matrix run 37098199031 reported a Volt template hover
as \`const tags: any\` instead of its pinned \`const tags: string[]\` contract.
The upstream binding is \`const tags = ref(['new']);\` with no authored import:
the owning Nuxt app supplies the real Vue function through its generated
declarations. The registry revision and original whole-file Git blob remain
recorded in a separate reduced regression fixture. The original hover golden,
fixture revision, fatal-diagnostic checks and all budgets stay unchanged.

Canonical hover queries previously supplied no discovered declaration references.
Diagnostics supplied every declaration discovered beneath the workspace, which
could also expose a sibling app's ambient globals. The editor mirror intentionally
replaces inherited file membership with its query surfaces, so selecting the
correct nearest tsconfig alone does not restore those declaration roots.

The bridge now selects physical discovered declaration files with the existing
TypeScript ownership authority and its own actual configuration. Editor mirror
construction and reference selection share the original workspace-root,
explicit-config and nearest-config selection. Project-reference resolution still
uses the original unique-owner implementation. Both canonical editor requests and
diagnostics receive the same selected declarations. Explicit unreadable configs
admit no globals; no-config workspace behavior remains scoped to existing
physical declaration files under that workspace. No owner is invented from a
package name or from equal source text.

Seven new Rust laws cover inherited hidden declarations, sibling exclusion,
solution references, explicit relative configs, unreadable configs, no-config
declaration families, fresh configuration reads and physical symlink identity.
Two genuine JSON-RPC cases use a real Vue package and native TypeScript backend:
the owning app must typecheck and return authored script/template hover ranges;
a sibling-only declaration must preserve the complete missing-ref diagnostic and
must not publish the Vue ref type. These Rust and server laws are **unexecuted**
in this private source draft and require exact-head Actions.

The new real-server test is registered in the existing explicit T1 runtime test
inventory. Pure corpus and runtime-candidate controls remain in T0. This adds no
job, pipeline stage, default route or validation waiver.

The new regression exposed a separate inherited TypeScript helper error:
\`Boolean(candidate)\` did not narrow nullable or false runtime candidates before
\`existsSync\`. An explicit nonempty string guard preserves selection for every
supported candidate. Two local physical-file laws cover absent/false/empty/missing
candidates and configured precedence. Those two laws, strict no-emit TypeScript
for the new files and selected lint pass. They grant no language-server or
ecosystem acceptance.

This private replay has authenticated main 7f1c7d4f as its direct source parent.
The reviewed f68 source and regression inputs remain unchanged. Configuration is
reread between calls; declaration selection and later mirror construction do not
form an atomic filesystem/configuration snapshot. No current-host or native File
authority is inferred from those reads. Complete existing-source Actions and
named Rust/server laws,
instruction ceilings and protected terminal merge remain future requirements.
No default native route, File admission, complete native responses, whole
fix-history acceptance or closure of #6883 is claimed. Other Matrix failures
remain unfinished.
