import assert from "node:assert/strict";
import { createRequire } from "node:module";
import test from "node:test";
void test(
  "actual native readonly primitive metadata needs zero literal member parses",
  {
    skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1",
  },
  () => {
    const binding = createRequire(import.meta.url)("@vizejs/native");
    assert.equal(typeof binding.analyzeSfc, "function");
    for (const count of [32, 128, 512]) {
      const props = Array.from({ length: count }, (_, index) => ({
        name: "p_" + index,
        type: "number",
        required: false,
      }));
      const source =
        '<script setup lang="ts">defineProps<{ ' +
        props.map(({ name }) => "readonly " + name + "?: number;").join(" ") +
        " }>()</script>";
      const { profile, ...actual } = binding.analyzeSfc(source, {
        filename: "Readonly" + count + ".vue",
        cache: false,
        profile: true,
      });
      assert.deepEqual(actual, { emits: [], props });
      assert.equal(
        profile.literalMemberParseCalls,
        0,
        "an actual baseline native image must fail this candidate counter contract",
      );
      assert.equal(profile.analyzeCalls, 1);
    }
    const source =
      '<script setup lang="ts">defineProps<{ readonly "scope.name"?: string; readonly café?: number; readonly complex?: number | string; readonly [key]: string; readonly [sideEffect()]: number }>()</script>';
    const { profile, ...actual } = binding.analyzeSfc(source, {
      filename: "ReadonlyRefusals.vue",
      cache: false,
      profile: true,
    });
    assert.deepEqual(actual, {
      emits: [],
      props: [
        { name: "scope.name", type: "string", required: false },
        { name: "café", type: "number", required: false },
        { name: "complex", type: "number | string", required: false },
      ],
    });
    assert.equal(profile.literalMemberParseCalls, 5);
  },
);
