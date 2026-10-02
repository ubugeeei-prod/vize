import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { moves } from "./profile-export-host-contract.ts";
import { prepare } from "./profile-export-host-preflight.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const mode = process.argv[2];
if (!["moves", "integrate", "check"].includes(mode))
  throw new Error("usage: move-profile-export-host.ts moves|integrate|check");
const states = moves.map(([before, after]) => {
  const old = fs.existsSync(path.join(root, before)),
    next = fs.existsSync(path.join(root, after));
  if (old === next) throw new Error("profile export move collision or missing source");
  return old;
});
if (states.some((old) => old !== states[0])) throw new Error("partial profile export move");
if (mode === "moves") {
  for (const [before, after] of moves) {
    if (fs.existsSync(path.join(root, before))) {
      fs.mkdirSync(path.dirname(path.join(root, after)), { recursive: true });
      fs.renameSync(path.join(root, before), path.join(root, after));
    }
  }
} else {
  if (states[0]) throw new Error("profile export must be moved before integration");
  const changes = prepare((file) => fs.readFileSync(path.join(root, file), "utf8"));
  if (mode === "check" && changes.size) throw new Error("profile export integration is incomplete");
  if (mode === "integrate")
    for (const [file, content] of changes) fs.writeFileSync(path.join(root, file), content);
}
