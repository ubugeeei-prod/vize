import { pathToFileURL } from "node:url";
import path from "node:path";
const context = JSON.parse(process.argv[2]);
const helper = (name: string) =>
  import(pathToFileURL(path.join(context.repoRoot, "tools/support/compat/npm", name)).href);
const { runInitTypecheckChecks } = await helper("smoke-release-init-typecheck.mjs");
const { runFreshProjectInitChecks } = await helper("smoke-release-init-fresh.mjs");
context.packed = new Map(Object.entries(context.packed));
context.compatiblePacked = new Set(context.compatiblePacked);
context.versions = new Map(Object.entries(context.versions));
runInitTypecheckChecks(context.installDir, context.vizeBin, context.repoRoot, context.peers);
runFreshProjectInitChecks(context);
