import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import path from "node:path";

import { buildUiResolverManifest, renderUiResolverManifest } from "./resolver-manifest.ts";

const root = process.cwd();
const target = path.join(root, "src/resolver/resolver-manifest.ts");
writeFileSync(target, renderUiResolverManifest(buildUiResolverManifest(root)));
// Keep the committed file in the repository format.
spawnSync("vp", ["fmt", "--write", target], { stdio: "inherit" });
process.stdout.write(`wrote ${path.relative(root, target)}\n`);
