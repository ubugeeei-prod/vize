# Installed batch storage ownership

The module-resolution root and the owner of mutable cache artifacts are separate
identities. Expanding the semantic root for relative workspace imports preserves
native package visibility: a sibling must not acquire an app-only package merely
because its projection became a descendant of the app's mirrored node_modules.

The CLI supplies the requested program root and original configuration path as
an explicit storage owner before scanning. Programs with an expanded root or
installed inputs use a separate private namespace, keyed by the canonical owner,
semantic root and original config. Ordinary single-root namespaces are unchanged;
embedding callers retain the existing installed-source lifetime unless they opt
in. Their plain build info may survive, while their generated mirror is cleared.

If a cache redirect would put the private namespace inside the semantic source
tree, the existing isolated installed-source lifetime is retained instead.
Persistent namespaces never reuse ordinary project mirrors. Their parent is
private to one owner, their context entries use the existing bounded Canon cache
maintenance. An owner's usage lease is acquired before its first owned directory
is created, so concurrent GC cannot remove it during initialization or race its
private permissions. Clean removes only the selected owner's storage. MaterializeLock
covers validation, publication and native checking. Before writing, every owned
path's directory ancestors are checked from the namespace down, without following
old links. Stale links are removed as links and recreated as owned directories.
Changed hardlinked leaves are replaced atomically, and native build-info outputs
are detached before Corsa can write them. Installed bytes and mtimes remain intact.

This repairs repeated-process storage reuse from #10934, reported by dmnlk.
It is a persistence contract, not evidence for the cold-check latency target.
The pinned negative/repair corpus, native package-visibility vector, three-process
CLI replay, unchanged mirror mtimes and linked-target invariants are required.
