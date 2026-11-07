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

export interface ReleaseZedProfile {
  userData: string;
  shim: string;
  installed: string;
}

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

/** Prepare the isolated profile that the packaged Zed smoke launches. */
export function prepareReleaseZedProfile(
  session: string,
  packaged: string,
  evidence: string,
): ReleaseZedProfile {
  const userData = path.join(session, "user-data");
  const shim = path.join(session, "bin");
  const installed = path.join(userData, "extensions", "installed", "vize");
  fs.mkdirSync(installed, { recursive: true });
  fs.mkdirSync(shim, { recursive: true });
  fs.cpSync(packaged, installed, { recursive: true });
  fs.writeFileSync(path.join(userData, "extensions", "installed", ".rescan"), "");
  writeReleaseZedSettings(path.join(userData, "config"), evidence);
  return { userData, shim, installed };
}
