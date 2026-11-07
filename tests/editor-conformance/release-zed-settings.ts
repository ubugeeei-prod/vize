import fs from "node:fs";
import path from "node:path";

const settings = {
  session: { trust_all_worktrees: true },
  languages: {
    "Vue.js": {
      language_servers: ["vize", "..."],
    },
  },
  auto_install_extensions: { vize: false, vue: false },
  auto_update_extensions: { vize: false, vue: false },
  telemetry: { diagnostics: false, metrics: false },
};

export interface ReleaseZedProfile {
  userData: string;
  shim: string;
  installed: string;
  vueLanguage: string;
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
  const extensions = path.join(userData, "extensions", "installed");
  const installed = path.join(extensions, "vize");
  fs.mkdirSync(installed, { recursive: true });
  fs.mkdirSync(shim, { recursive: true });
  fs.cpSync(packaged, installed, { recursive: true });
  const vueLanguage = installVueLanguageFixture(packaged, extensions, evidence);
  fs.writeFileSync(path.join(extensions, ".rescan"), "");
  writeReleaseZedSettings(path.join(userData, "config"), evidence);
  return { userData, shim, installed, vueLanguage };
}

/** Register the Vue.js language without starting a second language server. */
function installVueLanguageFixture(packaged: string, extensions: string, evidence: string): string {
  const vueLanguage = path.join(extensions, "vue");
  const language = path.join(vueLanguage, "languages", "vue");
  fs.mkdirSync(path.join(vueLanguage, "grammars"), { recursive: true });
  fs.cpSync(path.join(packaged, "languages", "art-vue"), language, { recursive: true });
  fs.copyFileSync(
    path.join(packaged, "grammars", "vue.wasm"),
    path.join(vueLanguage, "grammars", "vue.wasm"),
  );

  const packagedManifest = fs.readFileSync(path.join(packaged, "extension.toml"), "utf8");
  const grammar = packagedManifest.match(/\[grammars\.vue\][\s\S]*?(?=\n\[|$)/u)?.[0];
  if (!grammar) throw new Error("packaged Zed extension does not declare the Vue grammar");
  const manifest = [
    'id = "vue"',
    'name = "Vue language fixture"',
    'version = "0.0.0"',
    "schema_version = 1",
    'authors = ["Vize contributors"]',
    'description = "Release-smoke language identity for the packaged Vize server"',
    'repository = "https://github.com/ubugeeei-prod/vize"',
    'languages = ["languages/vue"]',
    "",
    grammar.trim(),
    "",
  ].join("\n");
  fs.writeFileSync(path.join(vueLanguage, "extension.toml"), manifest);

  const configPath = path.join(language, "config.toml");
  const artConfig = fs.readFileSync(configPath, "utf8");
  const vueConfig = artConfig
    .replace(/^name = "Art Vue"$/mu, 'name = "Vue.js"')
    .replace(/^path_suffixes = \["art\.vue"\]$/mu, 'path_suffixes = ["vue"]');
  if (vueConfig === artConfig) throw new Error("packaged Art Vue language contract changed");
  fs.writeFileSync(configPath, vueConfig);

  fs.mkdirSync(evidence, { recursive: true });
  fs.writeFileSync(path.join(evidence, "zed-vue-language-extension.toml"), manifest);
  fs.writeFileSync(path.join(evidence, "zed-vue-language-config.toml"), vueConfig);
  return vueLanguage;
}
