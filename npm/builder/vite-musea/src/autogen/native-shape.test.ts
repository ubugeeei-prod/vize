import test from "node:test";
import assert from "node:assert/strict";

import {
  fromNativeOutput,
  toNativeAutogenConfig,
  toNativePropDefinitions,
} from "./native-shape.js";

void test("native autogen mapper uses NAPI camelCase fields", () => {
  const [prop] = toNativePropDefinitions([
    {
      name: "variant",
      propType: "'primary' | 'secondary'",
      required: true,
      defaultValue: "primary",
    },
  ]);

  assert.deepEqual(prop, {
    name: "variant",
    propType: "'primary' | 'secondary'",
    required: true,
    defaultValue: "primary",
  });
  assert.equal("prop_type" in prop, false);
  assert.equal("default_value" in prop, false);

  assert.deepEqual(
    toNativeAutogenConfig({
      maxVariants: 3,
      includeDefault: false,
      includeBooleanToggles: false,
      includeEnumVariants: true,
      includeBoundaryValues: true,
      includeEmptyStrings: false,
    }),
    {
      maxVariants: 3,
      includeDefault: false,
      includeBooleanToggles: false,
      includeEnumVariants: true,
      includeBoundaryValues: true,
      includeEmptyStrings: false,
    },
  );

  assert.deepEqual(
    fromNativeOutput({
      variants: [
        {
          name: "Secondary",
          isDefault: false,
          props: { variant: "secondary" },
        },
      ],
      artFileContent: "<art />",
      componentName: "Button",
    }),
    {
      variants: [
        {
          name: "Secondary",
          isDefault: false,
          props: { variant: "secondary" },
          description: undefined,
        },
      ],
      artFileContent: "<art />",
      componentName: "Button",
    },
  );
});
