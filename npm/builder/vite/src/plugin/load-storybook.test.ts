import assert from "node:assert/strict";

import type { VizePluginState } from "./state.ts";
import { transformHook } from "./load.ts";

const storybookTsxTransform = await transformHook(
  {} as VizePluginState,
  `export const Example = () => <button>Story</button>;\n`,
  "/src/AfButton.stories.tsx",
  { ssr: false },
);

assert.equal(
  storybookTsxTransform,
  null,
  "Storybook TSX CSF files should stay on Vite's regular JSX pipeline",
);

const storybookJsxTransform = await transformHook(
  {} as VizePluginState,
  `export const Example = () => <button>Story</button>;\n`,
  "/src/AfButton.story.jsx",
  { ssr: false },
);

assert.equal(
  storybookJsxTransform,
  null,
  "Storybook JSX CSF files should stay on Vite's regular JSX pipeline",
);

console.log("vite-plugin-vize Storybook CSF load tests passed!");
