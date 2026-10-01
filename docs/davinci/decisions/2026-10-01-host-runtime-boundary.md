# Carton owns host runtime discovery (2026-10-01)

Decision for [#6834](https://github.com/ubugeeei-prod/vize/issues/6834), after
[#7298](https://github.com/ubugeeei-prod/vize/pull/7298) actually merged and
Issue #6833 closed. Refreshed main `cb806ff4fcbbfe93971a23007a9e62df2252e679`
contains retirement `d457f95dbc9c079c73919a258e0bbbb1cfe24e48`; its
[protected queue Check](https://github.com/ubugeeei-prod/vize/actions/runs/36790341706)
passed. Fresh Git transport and the GitHub API confirm the main ancestry.

Corsa executable discovery and transport classification are legacy host
integration. Move their five implementation/test files unchanged from L0 to
Carton in a move-only commit, then expose the same host modules there.
Discovery order, environment precedence, explicit-path errors, package
probing and wrapper normalization stay unchanged. The compile-time workspace
root still walks two parents from the manifest; both owner directories have
that depth. Existing resolver and transport laws follow their implementation.

Move the native-only `which` dependency to Carton. Carton also declares its
host JSON parser directly. Canon, Patina, Maestro and CLI declare the actual
host owner and import its Corsa modules. Remove Canon's private L0-as-Carton
alias so it cannot shadow that owner; its existing legacy storage imports
continue through Carton's identical L0 re-exports. Content mapping and native
storage stay directly on L0. No level takes a normal/build dependency back to
Carton, and no product route or fixed compilation/diagnostic bytes change.

The storage import laws permit only the two named host modules in the three
affected product trees; other products keep their existing strict prohibition.
Canon's content-mapper scan still rejects Carton storage imports. The independent
directory and level gates continue checking normal/build, optional, target and
transitive workspace edges without new exceptions. Plugin identity already
hashes Carton's source directory, so moved host edits remain observable.

Replay `python3 tools/support/levels/move-host-runtime.py moves`, commit
only the five moves, then run `integrate`, format, reconcile the existing
lockfile and regenerate the source inventory shards. Replaying must reject
missing sources, move collisions and an unmoved integration. Existing version
and dependency selections remain pinned.

This is a bounded host slice. Config, locale catalogues, profiler host facilities,
LSP coordinate protocols and platform portability remain open under #6834.
Higher native providers, L1 defaults, product fix-history gates and native-only
acceptance remain unfinished. Exact-head Actions and actual protected-queue
merge are required before reporting this slice complete.

GitHub authentication and Git transport were restored before publication.
The paired issue comment and independent PR carry this decision; exact-head
Actions and the actual protected-queue merge remain completion requirements.

Review corrections for #6832: the stage catalogue identifies L0 as `vize_l0`,
not its legacy Carton facade, and includes the actual L4 package. Registration
does not claim unfinished L4 emission complete. Consumer inventories retain
Carton mentions as legacy host/compatibility infrastructure, not L0 aliases.
Canon keeps explicit L0 storage imports and admits only the two named Corsa
host modules; it cannot restore its private L0-as-Carton alias.

Replay review: preserve complete grouped import items, including aliases, bare
modules and nested groups. The check phase rejects still-unmigrated grouped
host roots as well as direct paths. Seven regression laws compile representative
rewritten Rust, preserve unrelated storage groups and reject deliberately
unmigrated consumers; replaying the actual integrated tree changes no sources.

Replay and checking share an offset-preserving lexical view that excludes Rust
comments, raw/byte/C strings, quoted strings and character literals. Real
qualified host paths still migrate; import-like fixture data remains unchanged.
Every generated import retains the original outer attributes, including cfg
and cfg_attr. Disabled missing-symbol imports compile only if every split
keeps its conditions; integration/check fixtures verify literal preservation.
