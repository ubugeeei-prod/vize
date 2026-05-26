# vize_curator

`vize_curator` contains local-only inspection and reporting helpers for Vize.

The crate is not published. It is meant for developer workflows that package
compiler inspector payloads, agent-readable reports, graph metadata, and future
profiling artifacts without growing the public crate surface.

The playground uses this crate through the Vize WASM package so the browser and
CLI paths share the same native graph and diff logic. JavaScript remains the
orchestration layer for running `@vue/compiler-sfc` and formatting output.

## Current Entry Points

- `inspector::build_payload`
- `inspector::build_playground_url`
- `inspector::build_agent_report`
- `inspector::build_graph`
- `inspector::build_diff`

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
