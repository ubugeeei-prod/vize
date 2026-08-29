/** Compile-only assertions for the source-owned registry contract. */

import {
  UI_SOURCE_REGISTRY_PACKAGE_NAME,
  UI_SOURCE_REGISTRY_SCHEMA_VERSION,
  createUiSourceRegistryManifest,
  getUiSourceFamilyInfo,
  type UiSourceFamilyManifest,
  type UiSourceRegistryManifest,
  type UiSourceRegistryOutputFormat,
} from "./source-registry.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const manifest = createUiSourceRegistryManifest();
export const button = getUiSourceFamilyInfo("button", manifest);

type _ManifestSchemaVersionIsLiteral = Expect<
  Equal<typeof manifest.schemaVersion, typeof UI_SOURCE_REGISTRY_SCHEMA_VERSION>
>;
type _ManifestPackageNameIsLiteral = Expect<
  Equal<typeof manifest.packageName, typeof UI_SOURCE_REGISTRY_PACKAGE_NAME>
>;
type _ManifestShapeIsClosed = Expect<Equal<typeof manifest, UiSourceRegistryManifest>>;
type _InfoMayMiss = Expect<Equal<typeof button, UiSourceFamilyManifest | undefined>>;
type _OutputFormatsAreMachineReadable = Expect<
  Equal<UiSourceRegistryOutputFormat, "json" | "jsonl">
>;

if (button != null) {
  const sourceFiles: readonly `src/${string}`[] = button.source.sourceFiles;
  const behavior: `src/${string}.behavior.md` = button.source.behaviorContract;
  const rendererFixture: `${string}Consumer.vue` | `${string}.vue` | null =
    button.source.rendererFixture;
  void sourceFiles;
  void behavior;
  void rendererFixture;

  // @ts-expect-error manifest families are immutable to callers.
  button.name = "checkbox";
  // @ts-expect-error source files are immutable to callers.
  button.source.sourceFiles.push("src/checkbox.ts");
}
