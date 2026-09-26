import { describe, expect, it } from "vite-plus/test";
import { graphLineKinds, partitionKinds } from "./partition";

// Byte-for-byte the compiler's pages for `<div>{{ msg }}</div>` (the Rust
// `wasm::tests_spolvero` pins).
const GRAPH = `[s3-folio]
phase=built

[s3-folio.regions]
id=0 parent=- owner=- span=3:23
id=1 parent=0 owner=0 span=8:17

[s3-folio.ops]
id=0 kind=impeto.insert-node region=0 effect=- span=3:23
id=1 kind=impeto.set-text region=1 effect=0 span=8:17

[s3-folio.effects]
id=0 owner=1 region=1 span=8:17

`;
const PARTITION = `[s3-partition-folio]

[s3-partition-folio.ops]
op=0 kind=static span=3:23
op=1 kind=dynamic span=8:17

`;

describe("partition overlay", () => {
  it("reads the exported kind per op id", () => {
    expect([...partitionKinds(PARTITION)]).toEqual([
      [0, "static"],
      [1, "dynamic"],
    ]);
  });

  it("marks only op lines of the graph page, never region or effect ids", () => {
    expect([...graphLineKinds(GRAPH, partitionKinds(PARTITION))]).toEqual([
      [8, "static"],
      [9, "dynamic"],
    ]);
  });
});
