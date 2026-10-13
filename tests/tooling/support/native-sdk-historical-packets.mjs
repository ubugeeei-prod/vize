// This archive runs only nonexecuted parser fixture laws; no actual runtime is replaced.
import assert from "node:assert/strict";
import path from "node:path";
import { pathToFileURL } from "node:url";
import {
  archivedNativeSdkModule,
  nativeSdkSourceRoot,
  nativeSdkHistoricalView,
} from "./native-sdk-historical-source.mjs";

nativeSdkHistoricalView();
const sourceFs = new URL("./native-sdk-historical-source.mjs", import.meta.url).href;
const prefix = "tools/support/compat/github/";
const once = (text, before, after) => {
  assert.equal(text.split(before).length, 2, "one exact historical module relocation");
  return text.replace(before, after);
};
function moduleURL(file, dependencies) {
  const original = archivedNativeSdkModule(file);
  let source = original.body.toString();
  const replacements = [
    ['import fs from "node:fs";', `import fs from ${JSON.stringify(sourceFs)};`],
  ];
  for (const [before, after] of dependencies)
    replacements.push([JSON.stringify(before), JSON.stringify(after)]);
  for (const [before, after] of replacements) source = once(source, before, after);
  const originalURL = JSON.stringify(pathToFileURL(path.join(nativeSdkSourceRoot, file)).href);
  assert.equal(source.split("import.meta.url").length - 1, original.importMetaOccurrences);
  source = source.replaceAll("import.meta.url", originalURL);
  let restored = source;
  assert.equal(restored.split(originalURL).length - 1, original.importMetaOccurrences);
  restored = restored.replaceAll(originalURL, "import.meta.url");
  for (const [before, after] of replacements.toReversed()) restored = once(restored, after, before);
  assert.deepEqual(Buffer.from(restored), original.body, "whole historical module inverse");
  return "data:text/javascript;base64," + Buffer.from(source).toString("base64");
}
const compositionURL = moduleURL(prefix + "native-lexical-composition-custody.mjs", []);
const metadataURL = moduleURL(prefix + "native-lexical-dollar-metadata.mjs", [
  ["./native-lexical-composition-custody.mjs", compositionURL],
]);
const receiptURL = moduleURL(prefix + "native-lexical-dollar-receipt.mjs", [
  ["./native-lexical-dollar-metadata.mjs", metadataURL],
]);
const composition = await import(compositionURL);
const metadata = await import(metadataURL);
const receipt = await import(receiptURL);
export const { digest, cargoArgs, originalReceiptNames, unobserved, sourcePaths } = metadata;
export const { validateLexicalSourceBodies } = composition;
export const { nativeLaw, producerLaws, validateLexicalDollarPackets } = receipt;
