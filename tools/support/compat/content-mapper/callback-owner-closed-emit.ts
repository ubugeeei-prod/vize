/**
 * #12622: certify a module's listener gate block gaining the closed-SFC route.
 * Both blocks are rendered from the gate source itself (current, and its
 * predecessor through the closed-emit-factory custody record), exactly as
 * `emit_gate_aliases` writes them.
 */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { beforeClosedEmitFactory } from "./closed-emit-factory-transition.ts";

type Edit = { start: number; end: number; replacement: Buffer };
export const closedEmitGatePath =
  "crates/vize_canon/src/virtual_ts/scope/component_events/emit_props_gate.rs";
const shared = 'import("vize:virtual-ts").';

function rustConstant(source: string, name: string): string {
  const match = source.match(new RegExp(`const ${name}: &str = "((?:[^"\\\\]|\\\\.)*)";`, "u"));
  assert.ok(match, name);
  return match[1]!.replaceAll("\\n", "\n").replaceAll('\\"', '"');
}

/** The gate block of a standalone or hoisted module for one revision of the gate source. */
export function renderedGateBlock(source: string, hoisted: boolean): string {
  const gate = rustConstant(source, "GATE_ALIASES");
  const local = gate.includes("{closed_aliases}")
    ? rustConstant(source, "CLOSED_EMIT_ALIASES")
    : "";
  const name = (standalone: string, sdk: string) => (hoisted ? `${shared}${sdk}` : standalone);
  return gate
    .replaceAll("{closed_aliases}", hoisted ? "" : local)
    .replaceAll("{closed}", name("__vize_emit_closed", "$ClosedEmit"))
    .replaceAll("{closed_factory}", name("__vize_emit_closed_factory", "$ClosedEmitFactory"))
    .replaceAll("{emit_props_factory}", name("$EmitPropsFactory", "$EmitPropsFactory"));
}

let blocks: Array<{ before: string; after: string }> | undefined;
function gateBlocks(): Array<{ before: string; after: string }> {
  if (blocks) return blocks;
  const current = readFileSync(
    new URL(`../../../../${closedEmitGatePath}`, import.meta.url),
    "utf8",
  );
  const previous = beforeClosedEmitFactory(closedEmitGatePath, current);
  assert.notEqual(previous, current, "the closed emit factory record owns the gate source");
  blocks = [false, true].map((hoisted) => ({
    before: renderedGateBlock(previous, hoisted),
    after: renderedGateBlock(current, hoisted),
  }));
  return blocks;
}

/** The one gate block edit when `before` declares the predecessor block and `after` the current one. */
export function closedEmitGateEdits(before: string, after: string): Edit[] {
  for (const gate of gateBlocks()) {
    const at = before.indexOf(gate.before);
    if (at < 0 || after.split(gate.after).length !== 2) continue;
    assert.equal(before.indexOf(gate.before, at + 1), -1, "one gate block per module");
    const start = Buffer.byteLength(before.slice(0, at));
    const end = start + Buffer.byteLength(gate.before);
    return [{ start, end, replacement: Buffer.from(gate.after) }];
  }
  return [];
}
