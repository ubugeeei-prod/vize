## Root cause

An editor alias build (`build_with_import_facts`) records the filesystem inputs of every package-route resolution, so later lookups can check that the route still holds. `RouteDiscovery::resolve` appended each resolution's inputs to a `Vec<PathBuf>`:
- the lookup's consulted paths,
- the resolution settings' inputs,
- the importer context's inputs,
- the reachability scan's inputs.

The build then ran `sort()` + `dedup()` over the whole vector. Most of those paths repeat on every resolution. On a primevue-volt lifecycle session (local trace with a temporary counter), each build appended **126,014** paths for **5,685** distinct ones:

| source | paths appended per build |
| --- | ---: |
| consulted | 57,513 |
| settings | 536 |
| context | 338 |
| reachability | 67,627 |

Every one of them was cloned, and the 126 k-entry vector was sorted path by path. A local `sample` attributed 1,077 of 7,593 `prepare_hosts_cached` samples (14 %) to that one sort (`build.rs:338`).

## Fix

`RouteDiscovery` records inputs into an `FxHashSet<PathBuf>`. A path is cloned only the first time it arrives. The build then sorts the distinct paths once, so `route_inputs` is the same sorted, deduplicated list as before.

## Evidence

- `route_inputs` is unchanged by construction: sorted unique paths either way. The `class_component_route_inputs` snapshot still passes.
- **Tests:** `cargo test -p vize_canon --lib vue_dependencies_alias`: 62 passed. `--lib corsa_bridge` fails three tests with this change, and the base commit fails them too:
  - two `vue_document::build` tests fail on macOS `/private/var` tempdir spelling (checked on the base),
  - one `original_program` test needs a native tsgo,
- **Lint:** `cargo clippy -p vize_canon --lib -- -D warnings -D clippy::wildcard_imports` is clean.
- **Blacksmith:** dispatched once the merge queue drains.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
