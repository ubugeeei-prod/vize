import { defineConfig } from "vite-plus";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

/** Run `pnpm gallery:globals` to try project-wide brand, theme and locale controls. */
export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["src/**/*.vue"],
      inlineArt: true,
      previewSetup: "musea.preview.ts",
      toolbar: [
        {
          id: "brand",
          title: "Brand",
          type: "select",
          options: ["default", "ocean"],
          default: "default",
        },
        {
          id: "scheme",
          title: "Component theme",
          type: "toggle",
          default: "light",
          options: [
            { value: "light", label: "Light", icon: "☀" },
            { value: "dark", label: "Dark", icon: "☾" },
          ],
        },
        { id: "locale", title: "Locale", type: "select", options: ["en", "ja"], default: "en" },
      ],
    }),
  ],
});
