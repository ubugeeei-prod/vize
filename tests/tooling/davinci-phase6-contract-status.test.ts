import { registerRecutChecks } from "./support/davinci-recut.ts";

registerRecutChecks({
  phase: 6,
  strictDisjointPaths: true,
  taskFiles: ["phase-6-tasks.md", "phase-6-tasks-later.md"],
  predecessorFiles: ["phase-3.md", "phase-4.md", "phase-5.md"],
  provisionalGateLines: [
    "- [ ] TS-48..51 green; contracts documented with semver policy",
    "- [ ] MoonBit + Volt + JS-rule validations complete; `ExprRef` report closed",
    "- [ ] Completion metrics reconciled; v1 package delivered",
  ],
});
