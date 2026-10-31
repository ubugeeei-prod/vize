import { readFileSync, realpathSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import type { Recorder } from "../../config/windows_consumer/common.ts";
import { captureDeclarationGraph } from "../computed_event_custody.ts";
import { assertGraph, type Role } from "./contract.ts";

/** Keep the actual graph and every declaration before a role comparison can fail. */
export function installedGraph(cliRoot: string, role: Role, record: Recorder) {
  const root = realpathSync(cliRoot);
  record.json("installation-phase.json", { phase: "capture-declaration-graph" });
  const graph = captureDeclarationGraph(root);
  record.json("installed-declaration-graph.json", { role, root, graph });
  for (const file of graph.files)
    writeFileSync(
      join(
        record.root,
        "sdk--" + relative(root, file.path).replaceAll("\\", "/").replaceAll("/", "--"),
      ),
      readFileSync(file.path),
      { flag: "wx" },
    );
  record.json("installation-phase.json", { phase: "validate-declaration-graph" });
  assertGraph(role, root, graph);
  return graph;
}
