import { registerRecutChecks } from "./support/davinci-recut.ts";

registerRecutChecks({
  phase: 5,
  taskFiles: ["phase-5-tasks.md", "phase-5-tasks-later.md"],
  predecessorFiles: ["phase-3.md", "phase-4.md"],
  provisionalGateLines: [
    "- [ ] TS-42 incremental≡clean green; TS-43 key stability green",
    "- [ ] TS-44 latency/RSS/idle budgets green on large projects; TS-45 conformance green",
    "- [ ] TS-46 adoption accounting; TS-47 fault tolerance",
    "- [ ] #698/#699 closed; `parse_sfc`-per-request pattern gone (grep ceiling: 0 request-path sites)",
  ],
});
