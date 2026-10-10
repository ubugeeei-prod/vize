# Document project ownership in Maestro (#8371)

One editor workspace may contain independent Vite packages or TypeScript
projects. A request resolves the closest package, Vite/dedicated config, or
TypeScript/JavaScript project boundary, stopping at the enclosing editor folder.
A sibling package does not inherit another package's policy.

The primary project retains its existing state. Additional document targets
lazily create one cached state per boundary, with independent config, virtual
sources, Canon/Corsa owners, request locks, and derived caches. All states share
the authoritative editor document store and its source revisions. Contexts refer
weakly to the registry so importer diagnostics can route back to their actual
owner without ownership cycles. Per-document flags gate every routed provider.

Host evaluation produces one ConfigDocument for both linter plans and editor
settings. Initialization options are retained and applied last to every context.
Vite-owned checker paths are projected against Vite's selected source root.
Dedicated-file relative checker paths resolve against that config file;
explicit CLI `--tsconfig` continues to resolve against the invocation directory.

Capability advertisement covers the existing profile and the recommended
provider union for multi-root workspaces, package workspaces, and TypeScript
reference manifests, including JSONC. Explicit initialization options are
applied to that union last. Advertisement does not instantiate extra project
states or start Corsa sessions. Single-project dedicated formatting stays opt-in.
pnpm workspace manifests also advertise this provider union.

The existing bounded diagnostic worker dispatches to the owning context. No
worker is added per project. Watched config changes invalidate routing, replace
only affected cached contexts, and permanently retire their previous owners.
Old and queued native requests refuse results from retired owners. Filesystem
changes are forwarded to initialized contexts; unused packages remain idle.
Affected open URI/version pairs refresh through replacement owners when only
a configuration changes. Cross-package dependency refresh unions the reverse
indexes of initialized owners, then dispatches each importer to its own owner.
Workspace-folder removal retires cached contexts outside the remaining roots.
Shutdown retires all cached native owners.

Creating a new nested config marker refreshes affected open documents even when
no child context existed before that event. Removing the marker returns them to
the nearest remaining boundary, rebuilding from the latest unsaved buffer.
Retirement also rejects unstamped parser/lint/empty diagnostic packets and
initial synchronous feedback, before notification or lint-hover cache mutation.
The protocol control covers creation and removal without a document request;
the deterministic transport control covers an old lint/empty packet after the
replacement owner has published the same document version.

Project ignores exclude CLI discovery and LSP lint policy. An explicitly opened
file still receives parser and type diagnostics, preserving editor behavior.

The registry holds only package-local initialization cells. Node evaluation and
virtual-source warmup run outside its shared mutex and evaluate each generation
once. Retirement first marks a generation unavailable, including pending
initialization; native cleanup then runs outside the registry lock. A delayed
initializer cannot publish a retired owner into its replacement generation.

The public JSON-RPC fixture uses two packages with different quote policies and
hover flags, identical aliases resolving to different native types, custom
relative tsconfigs, and an explicit Vite source root. It covers concurrent
requests, explicit initialization overrides, config reload, positive/negative
diagnostics, and stable config evaluation counts across warm requests. It
prints cold/warm formatting timings for hosted observation without changing any
existing performance budget. State tests cover cache identity, retirement,
nearest boundaries, and lazy capability advertisement.

Global `workspace/symbol` scans the shared workspace inventory once, filtering
matched sources through their current project policy before ranking and the
global limit. Unmatched packages require no config evaluation. File rename uses
the current initialized native owners in parallel and one authored importer
scan, applying each importer's own aliases and provider flag. Exact duplicate
edits are removed; disabled package edits never reach the response.

File creation/deletion invalidates initialized owners and refreshes open typed
documents, including unresolved imports without a reverse edge. This fallback
is bounded by editor-open buffers. Physical rename moves the authoritative
buffer once and rebuilds it through the destination owner, retaining its version
and unsaved content. The workspace inventory outlives primary runtime retirement
and never borrows that retired runtime's native transaction.

Bulk requests capture stable routing generation and authored source revision.
Active config/folder mutations refuse requests before work begins; changes
during work refuse the aggregate reply. Individual native queries additionally
hold their own owner's request scope. The routing mutation guard spans folder
updates, owner retirement, route clearing, and virtual-source cleanup.

Public JSON-RPC controls reload the primary config before querying complete
symbol and import-edit vectors, preserving sibling package disables and explicit
initialization disables. Genuine Corsa controls then move an open source across
packages without another didOpen, delete its alias target, and recreate it,
requiring complete TS2322, TS2307, and clean notifications at the retained URI
and version. Unit controls cover lazy primary replacement, global filtering
before the result limit, and request refusal during and across routing mutation.

Synchronous lazy config evaluation still needs a genuine transport control for
a slow cold package alongside a warm unrelated request. Package-local locking
proves registry isolation; it alone does not prove executor freedom.

Current-source Actions, the protected queue, actual merge, and installed release
replay remain required. Source preparation alone does not close #8371.
