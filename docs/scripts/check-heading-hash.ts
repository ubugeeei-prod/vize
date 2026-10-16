import path from "node:path";
import { qualifyHeadingHashes } from "./heading-hash-qualifier.ts";

const output = path.resolve(import.meta.dirname, "../../docs-render-evidence/heading-hashes");
const site = path.resolve(import.meta.dirname, "../dist");
const receipt = await qualifyHeadingHashes(output, site);
console.log(
  `Verified ${receipt.observations.length} real heading hash flows with content-hashed runtime assets`,
);
