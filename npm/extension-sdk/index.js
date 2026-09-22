// Handshake constants of the `vize:contracts` package this SDK version binds.
// `tests/tooling/davinci-extension-sdk.test.ts` pins them to the released
// contract surface and to the Rust SDK.

export const PACKAGE = "vize:contracts@0.1.0";
export const PROTOCOL_VERSION = 1;
export const S1_PAGE_SCHEMA = 1;
export const S2_PAGE_SCHEMA = 1;
export const REQUIRED_FEATURES = Object.freeze(["s1-page@1", "s2-page@1"]);

/**
 * The capability offer for a guest lowering the given `lang` values: sorted
 * and unique, as the host requires.
 *
 * @param {readonly string[]} langs
 * @returns {import("./index.d.ts").Capability}
 */
export function capability(langs) {
  const features = [...new Set([...langs.map((lang) => `lang:${lang}`), ...REQUIRED_FEATURES])];
  features.sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
  return { protocolVersion: PROTOCOL_VERSION, features };
}
