# vize_curator

`vize_curator` contains local-only inspection and reporting helpers for Vize.

The crate is not published. It is meant for developer workflows that package
compiler inspector payloads, agent-readable reports, graph metadata, and future
profiling artifacts without growing the public crate surface.

## Current Entry Points

- `inspector::build_payload`
- `inspector::build_playground_url`
- `inspector::build_agent_report`
- `inspector::build_graph`

## Agent Reports

`vize inspector --format agent` uses this crate to print a JSON report with:

- the exact playground payload
- a ready-to-open playground URL
- summary metrics for files and options
- a lightweight cross-file graph extracted from local imports

The report is intended for local debugging and AI-agent handoff, not as a
stable public interchange format.
