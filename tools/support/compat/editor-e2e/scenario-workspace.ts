import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { prepareRealVueWorkspace } from "./real-vue-workspace.ts";

export interface ScenarioWorkspace {
  readonly root: string;
  readonly workspace: string;
}

/** Materialize the shared real-Vue workspace inside an owned temporary root. */
export function createScenarioWorkspace(prefix: string): ScenarioWorkspace {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), `${prefix}-`));
  const workspace = path.join(root, "real-vue");
  const initializedWorkspace = process.env.VIZE_E2E_WORKSPACE?.trim();
  if (initializedWorkspace) fs.cpSync(initializedWorkspace, workspace, { recursive: true });
  else prepareRealVueWorkspace(workspace);
  return { root, workspace };
}

export function removeScenarioWorkspace({ root }: ScenarioWorkspace): void {
  fs.rmSync(root, { force: true, recursive: true });
}
