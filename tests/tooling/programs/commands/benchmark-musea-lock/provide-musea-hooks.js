import fs from "node:fs";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
export function musea() {
  let root = "";
  return [
    {
      name: "vite-plugin-musea",
      config(userConfig) {
        return {
          build: {
            rollupOptions: {
              input: {
                "musea-static-entry": userConfig.build.rollupOptions.input,
                "musea-static-runtime": "virtual:musea-static-runtime",
              },
            },
          },
        };
      },
      configResolved(config) {
        root = config.root;
      },
      options() {
        return null;
      },
      async buildStart() {
        require("@vizejs/native");
        const ready = process.env.VIZE_TEST_MUSEA_HOLD_READY;
        const release = process.env.VIZE_TEST_MUSEA_HOLD_RELEASE;
        if (ready && release && !fs.existsSync(ready)) {
          fs.writeFileSync(ready, "ready");
          while (!fs.existsSync(release)) await delay(10);
        }
        fs.readdirSync(root).filter((name) => name.endsWith(".art.vue"));
      },
      resolveId(id) {
        return id.endsWith(".art.vue") ? "\0fixture:" + id : null;
      },
      load(id) {
        return id.startsWith("\0") ? "export default " + JSON.stringify(id) : null;
      },
      transform(code) {
        return { code };
      },
    },
  ];
}
