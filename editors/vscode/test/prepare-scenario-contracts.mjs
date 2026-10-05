import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const corpus = fileURLToPath(
  new URL(
    "../../../tests/_fixtures/differential/lsp-regressions/contract-before-startup-8012/",
    import.meta.url,
  ),
);

// Contract hover describes an existing imported component. Create that exact
// project topology before the real client registers file watchers.
export function prepareScenarioContracts(workspacePath) {
  fs.mkdirSync(path.join(workspacePath, "src"), { recursive: true });
  for (const file of ["ContractChild.vue", "ContractHost.vue"]) {
    fs.copyFileSync(path.join(corpus, `${file}.txt`), path.join(workspacePath, "src", file));
  }
}
