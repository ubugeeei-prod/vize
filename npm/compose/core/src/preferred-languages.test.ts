import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { usePreferredLanguages } from "./preferred-languages.ts";
import type { PreferredLanguagesHost } from "./preferred-languages.ts";

class LanguagesWindow extends EventTarget implements PreferredLanguagesHost {
  readonly navigator: { languages: readonly string[]; language?: string } = {
    languages: ["ja-JP", "en"],
  };
}

void test("never reads the server runtime navigator without a window", () => {
  assert.equal(typeof globalThis.window, "undefined");
  assert.deepEqual(usePreferredLanguages({ ssrLanguages: ["fr"] }).value, ["fr"]);
  assert.deepEqual(usePreferredLanguages().value, ["en"]);
});

void test("tracks languagechange and falls back to the single language", () => {
  const host = new LanguagesWindow();
  const scope = effectScope();
  const languages = scope.run(() => usePreferredLanguages({ host }));
  assert.deepEqual(languages?.value, ["ja-JP", "en"]);

  host.navigator.languages = [];
  host.navigator.language = "de";
  host.dispatchEvent(new Event("languagechange"));
  assert.deepEqual(languages?.value, ["de"]);

  scope.stop();
  host.navigator.languages = ["es"];
  host.dispatchEvent(new Event("languagechange"));
  assert.deepEqual(languages?.value, ["de"]);
});
