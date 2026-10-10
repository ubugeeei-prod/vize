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
settings. The existing Carton LSP snapshot owns the checked evaluation, selected
project root, raw ordered lint scopes and authored checker values; Maestro keeps
its existing snapshot facade call and never creates a project model in this host.
Malformed input returns an invalid snapshot before any editor policy changes.
An existing editor workspace with no config is a valid fresh project; the shared
checked loader still rejects an absent explicit CLI configuration selection.
The same workspace matcher host admits only the exact `ProjectIgnoreSet` and
`LintPlanScope` import pair for Vite global ignores. Its original single-symbol
historical positive, surplus/grouped/wildcard/other-host negative controls and
the unchanged 59-write path replay remain intact. Initialization options are retained and applied last to every context.
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
Active config/folder or authored-buffer mutations refuse requests before work begins; changes
during work refuse the aggregate reply. Individual native queries additionally
hold their own owner's request scope. The routing mutation guard spans folder
updates, owner retirement, route clearing, and virtual-source cleanup.
Edit deduplication uses typed hash identities for URI, range, text, and annotation
ID, retaining original order and distinct annotations without quadratic scans.

Public JSON-RPC controls reload the primary config before querying complete
symbol and import-edit vectors, preserving sibling package disables and explicit
initialization disables. Genuine Corsa controls then move an open source across
packages without another didOpen, delete its alias target, and recreate it,
requiring complete TS2322, TS2307, and clean notifications at the retained URI
and version. Unit controls cover lazy primary replacement, global filtering
before the result limit, and request refusal during and across routing mutation.

Foreground dispatch awaits one shared initialization future per boundary and
generation. Two lazily started process-wide config workers consume a bounded
async queue; no document creates its own thread. Warm contexts reuse their
initialized state immediately. Config evaluation and virtual-source warmup
remain one operation. Workspace symbol collection retains its existing worker;
manual rename scanning uses the bounded config workers.
Existing synchronous bulk traversal workers retain package-local once cells;
the queue limit counts submitted jobs, not their existing traversal threads.
Queue admission holds its mutex only while polling. A stored, canceled shared
initializer cannot retain that lock while waiting for capacity; popping a job
wakes all capacity waiters. A saturated-queue control leaves one admission
future pending and proves a fresh admission succeeds after a slot is freed.

The shared future and queued job capture weak context/registry references,
avoiding cycles when a request is canceled before consuming its result. A
running initializer holds its owner only while evaluating that config. Shutdown
retires all generations; queued work skips retired cells, and late results are
discarded. Worker failure logs and uses the existing synchronous configuration
path so an unrelated fallback profile never replaces the target's policy.

Reverse importer discovery retains all initialized, live owner indexes even
when native type checking is disabled. Each discovered importer is refreshed
through its own current policy. This preserves bookkeeping and lint/parser
refreshes without starting disabled Corsa sessions.

Open/change/close/rename record the authoritative source mutation before awaiting the
project, then prepare only the current version through the resolved owner.
This preserves same-document notification order while a config import is slow;
a canceled or superseded opener cannot restore its previous text.
Rename cleans initialized old caches without evaluating their configs, then
prepares its destination asynchronously. Native import edits resolve ownership
asynchronously too.

Boundary and config-watch matching resolve physical file identities, including
the nearest existing ancestor of new buffers. Registered workspace spelling
selects one cache key; notifications retain the editor's authored URI. An actual
rename control exposed incorrect root-policy fallback between `/private/var`
and `/var` spellings before this correction. Symlink/new-buffer and config-watch
controls preserve the same package owner and retire it on an aliased watch.

The genuine JSON-RPC latch control first failed because a cold package's Node
import blocked a warm sibling's completion. With async dispatch, the sibling's
whole completion envelope arrives before releasing the import; pending requests
then format the latest unsaved source and reuse one evaluation. Additional
transport controls require shutdown before releasing Node and retain later
destination edits when moving an open buffer into a cold package. State
controls prove eight concurrent targets share one owner, 48 queued jobs finish
with at most two active workers, and a canceled pending initializer retains no
registry/owner cycle after shutdown and probe completion. These are observed
correctness and fairness laws; hosted timings remain separate from the 10x
type-checker performance target.

Current-source Actions, the protected queue, actual merge, and installed release
replay remain required. Source preparation alone does not close #8371.

## Physical declaration aliases

The unchanged global tag-name law exposed a real macOS alias bug: discovery
canonicalized `/var` to `/private/var`, while editor buffers retained authored
URIs. One declaration was read from both disk and its open unsaved alias.
Join open declaration URIs to one physical cache key before scanning, selecting
the highest process-global document revision when two aliases are open. Keep
editor URIs, disk stamps, close/reopen authority and the 4 MiB limit unchanged.
One canonical call is admitted at the same existing physical host; all other
caller tuples and the original 59-write replay remain unchanged, with surplus
canonical and other-API calls still rejected. Whole source and protected
qualification remain required; original expected names are unchanged.

The genuine parent refresh preserves malformed dedicated configuration fallback:
workspace owners use one checked project snapshot and return before applying
fresh defaults on failure. Direct editor snapshots carry validity from the same
checked evaluation; trusted document projections preserve that status. Both
original direct-loader and additive workspace-owner whole feature controls apply.

Actual source Actions at `2c70` exposed a compatibility regression in the
unchanged complete module-link law: a dedicated config's authored relative
`tsconfig` had been replaced with an absolute path. Keep the raw checker values
and their separate selected load origin together when installing or reading a
native settings snapshot. Only the two existing batch/Corsa project-model
callers resolve that origin for execution; the module-link snapshot retains its
existing raw values and workspace-root path policy. The original whole law,
concurrent complete-snapshot control, and actual relative-config JSON-RPC
vectors must all pass. No historical oracle or provider budget changes.

The final hosted native proof uses `project-config-native.yml`: pin the actual PR head, build the production CLI, retain its existing source receipt, and run the entire unchanged `project_config_cli` target with required Corsa and decoded JSON-RPC/terminal capture. Ordinary PR Rust shards intentionally disable native execution and cannot supply this evidence. Keep their policy and the separate immutable native-phase recipe unchanged. All thirteen original nested lifecycle cases and sixteen captured editor processes must join the pinned source, post-test production executable, successful terminal and joined readers; complete decoded protocol and stderr hashes remain available. Preserve the canonical build receipt and record the actual locked build/test arguments separately. Original formatting and explicit controls must run; historical local captures remain historical.

The first hosted native run genuinely passed all 32 original target cases, but
Cargo test relinked the CLI and the retained binary join correctly refused
qualification. Prepare the exact Cargo-authored test target/package/source
artifact first, then build and receipt the canonical production CLI, and run
only the authenticated harness directly. Retain both harness hashes, raw
stdout/stderr, exact locked preparation/build and direct execution arguments,
and the unchanged post-execution CLI join. Reset only the Actions-owned scratch
evidence directory before each attempt so cached captures cannot count as new.

Actual source Actions found project consumers importing host snapshots outside
their existing physical owners. Keep the checked snapshot in the existing
config host and pass its L0 document, source path and project root by reference.
The workspace-folder host still evaluates once; its exact snapshot successor
retains the historical loader positive and rejects surplus or wrong-host APIs.
Share only the final normalizer from the existing rename path owner, retaining
its strict broken-link branch and the project walk's missing-ancestor behavior.
The complete 59-write path replay and sole normalization tuple stay unchanged.
The whole Vue-version getter moves into existing config reads without changing
its body, cfg attributes or visibility. Original native vectors still apply.
