import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const root = process.env.VIZE_TEMPLATE_EMIT_CAPTURE;
assert.ok(root);
const directory = readdirSync(root, { withFileTypes: true }).find((entry) => entry.isDirectory());
assert.ok(directory, "expected a captured TypeScript oracle");
const packet = JSON.parse(readFileSync(join(root, directory.name, "oracle.json"), "utf8")) as {
  nativeBinary?: string;
};
assert.ok(typeof packet.nativeBinary === "string" && packet.nativeBinary);
process.stdout.write(packet.nativeBinary);
