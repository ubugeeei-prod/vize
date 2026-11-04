import assert from "node:assert/strict";
import { test } from "node:test";
import { semanticFiles, semanticPacket } from "./template-alias-paired-11822-semantics.ts";

void test("independent template/directive anchors reject missing, suppressed, duplicate or shifted diagnostics", () => {
  const files = semanticFiles();
  assert.equal(files.size, 4);
  assert.equal(files.get("Negative.vue")!.split("\n")[4], "  {{ value.missing }}");
  assert.equal(files.get("Unused.vue")!.split("\n")[4], "  <!-- @vue-expect-error -->");
  const output =
    "src/Negative.vue(5,12): error TS2339: Property 'missing' does not exist on type '{ n: number; }'.\nsrc/Unused.vue(5,8): error vize4: Unused '@vue-expect-error' directive\n";
  const observed = semanticPacket(2, output, "", "/workspace");
  assert.equal(observed.packet.length, 2);
  assert.deepEqual(
    observed.packet.map(({ namespace, code }) => ({ namespace, code })),
    [
      { namespace: "TS", code: 2339 },
      { namespace: "vize", code: 4 },
    ],
  );
  assert.equal(observed.stdout, output.trimEnd());
  assert.throws(() => semanticPacket(2, output.replace("vize4", "TS4"), "", "/workspace"));
  assert.throws(() => semanticPacket(2, output.replace("TS2339", "vize2339"), "", "/workspace"));
  assert.throws(() => semanticPacket(2, output.replace("vize4", "other4"), "", "/workspace"));
  assert.throws(() => semanticPacket(0, output, "", "/workspace"));
  assert.throws(() => semanticPacket(1, output, "", "/workspace"));
  assert.throws(() => semanticPacket(2, output.split("\n")[0], "", "/workspace"));
  assert.throws(() => semanticPacket(2, output + output, "", "/workspace"));
  assert.throws(() =>
    semanticPacket(2, output.replace("Negative", "Suppressed"), "", "/workspace"),
  );
  assert.throws(() => semanticPacket(2, output.replace("(5,12)", "(3,12)"), "", "/workspace"));
});
