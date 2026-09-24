import path from "node:path";
import { fileURLToPath } from "node:url";

import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";
import { defineConfig } from "vite-plus";

// Musea gallery for every @vizejs/ui family. Stories are generated from each
// family's `examples/*.vue` by npm/ui/scripts/generate-gallery.ts (run by
// `pnpm gallery:ui`) and import the headless sources directly.
const uiRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../npm/ui");
const storiesRoot = path.join(uiRoot, "gallery", "stories");

export default defineConfig({
  root: path.dirname(fileURLToPath(import.meta.url)),
  plugins: [
    vize(),
    musea({
      include: [`${storiesRoot.split(path.sep).join("/")}/**/*.art.vue`],
      basePath: "/__musea__",
      inlineArt: false,
    }),
  ],
  server: {
    fs: { allow: [uiRoot, path.dirname(fileURLToPath(import.meta.url))] },
  },
  build: {
    outDir: "dist-ui-gallery",
    emptyOutDir: true,
  },
});
