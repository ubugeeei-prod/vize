import path from "node:path";
import { snapshot } from "node:test";

snapshot.setResolveSnapshotPath((testFile) =>
  path.join(path.dirname(testFile), "__snapshots__", `${path.basename(testFile)}.snap`),
);
