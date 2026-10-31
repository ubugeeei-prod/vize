import path from "node:path";
import { snapshot } from "node:test";

snapshot.setResolveSnapshotPath((testFile) => {
  if (testFile === undefined) throw new Error("Rspack snapshots require a test file");
  const name = path.basename(testFile);
  const suffix = name === "jsx.test.ts" ? ".child-values-6890.snap" : ".snap";
  return path.join(path.dirname(testFile), "__snapshots__", `${name}${suffix}`);
});
