# vize_curator

`vize_curator` contains local-only inspection and reporting helpers for Vize.

The crate is not published. It is meant for developer workflows that package
compiler inspector payloads, agent-readable reports, graph metadata, and profile
report rendering without growing the public crate surface.

The playground uses this crate through the Vize WASM package so the browser and
CLI paths share the same native graph and diff logic. JavaScript remains the
orchestration layer for running `@vue/compiler-sfc` and formatting output.

## Current Entry Points

- `inspector::build_payload`
- `inspector::build_playground_url`
- `inspector::build_agent_report`
- `inspector::build_graph`
- `inspector::build_diff`
- `profile::render_profile_report`
- `profile::print_profile_report`

## Agent Reports

`vize inspector --format agent` uses this crate to print a JSON report with:

- the exact playground payload
- a ready-to-open playground URL
- summary metrics for files and options
- a lightweight cross-file graph extracted from local imports

The same graph metadata is available in the playground inspector. Component
edges are added when a local Vue import is used as a template tag, which makes
batch payloads easier to inspect without reimplementing graph extraction in
TypeScript.

The report is intended for local debugging and AI-agent handoff, not as a
stable public interchange format.

## Profile Reports

`vize build --profile`, `vize lint --profile`, `vize fmt --profile`, and
`vize check --profile` all render their terminal reports through
`profile::print_profile_report`.

The low-level profiler remains in `vize_carton` because it is shared by parser,
compiler, linter, formatter, and type-checker crates. Curator owns the local CLI
report shape so profiling output can evolve with inspector and agent-facing
artifacts without making those crates depend on a developer-only package.
