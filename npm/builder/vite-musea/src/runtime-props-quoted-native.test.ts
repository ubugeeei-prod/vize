import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import path from "node:path";
import test from "node:test";
import type { NativeBinding } from "./native-loader.ts";
import { canonicalRuntimeComponent, runtimeComponent } from "./runtime-props-fixtures.ts";

void test(
  "real native metadata uses cooked runtime option names and refuses executable keys",
  {
    skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1",
  },
  async () => {
    const output = fileURLToPath(
      new URL("../../../../artifacts/musea-native-runtime-props", import.meta.url),
    );
    await mkdir(output, { recursive: true });
    const observed: Record<string, unknown> = { canonicalRuntimeComponent, runtimeComponent };
    try {
      const binding = createRequire(import.meta.url)("@vizejs/native") as NativeBinding;
      assert.equal(typeof binding.analyzeSfc, "function");
      assert.ok(binding.analyzeSfc);
      const plain = binding.analyzeSfc(canonicalRuntimeComponent, { filename: "Canonical.vue" });
      const quoted = binding.analyzeSfc(runtimeComponent, { filename: "Quoted.vue" });
      observed.plain = plain;
      observed.quoted = quoted;
      assert.notEqual(runtimeComponent, canonicalRuntimeComponent);
      assert.ok(runtimeComponent.includes(String.raw`["ty\u0070e"]`));
      assert.ok(runtimeComponent.includes(String.raw`"requ\u0069red"`));
      assert.deepEqual(quoted, plain);
      assert.equal(quoted.props.find((prop) => prop.name === "label")?.default_value, "true");
      assert.equal(quoted.props.find((prop) => prop.name === "count")?.required, true);
      const refusalSource = `<script setup lang="ts">defineProps({
      dynamic: { [typeKey]: String, [requiredKey]: true, [defaultKey]: "invented" },
      called: { [getType()]: Number, [getRequired()]: true, [getDefault()]: 1 },
      concatenated: { ["ty" + "pe"]: Boolean, ["requ" + "ired"]: true, ["def" + "ault"]: false },
    })</script>`;
      const refused = binding.analyzeSfc(refusalSource, { filename: "Refusal.vue" });
      observed.refusalSource = refusalSource;
      observed.refused = refused;
      assert.deepEqual(refused, {
        props: ["dynamic", "called", "concatenated"].map((name) => ({
          name,
          type: "unknown",
          required: false,
        })),
        emits: [],
      });
    } finally {
      await writeFile(path.join(output, "quoted-packets.json"), JSON.stringify(observed, null, 2));
    }
  },
);
