import fs from "node:fs";
import path from "node:path";

const settings = {
  session: { trust_all_worktrees: true },
  languages: {
    "Vue.js": {
      language_servers: ["vize", "..."],
    },
  },
  auto_install_extensions: { vize: false },
  auto_update_extensions: { vize: false },
  telemetry: { diagnostics: false, metrics: false },
};

/** Write the isolated Zed profile and retain the exact settings as release evidence. */
export function writeReleaseZedSettings(config: string, evidence: string): void {
  fs.mkdirSync(config, { recursive: true });
  fs.mkdirSync(evidence, { recursive: true });
  fs.writeFileSync(path.join(config, "settings.json"), `${JSON.stringify(settings)}\n`);
  fs.writeFileSync(
    path.join(evidence, "zed-settings.json"),
    `${JSON.stringify(settings, null, 2)}\n`,
  );
}
