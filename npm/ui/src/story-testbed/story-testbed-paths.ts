type SourceDirectory = "src" | `src/${string}`;

export interface StoryTestbedPathTarget {
  readonly canonicalName: string;
  readonly targetFiles: readonly `src/${string}`[];
}

function sourceDirectoryFor(file: `src/${string}`): SourceDirectory {
  const separator = file.lastIndexOf("/");
  return (separator === -1 ? "src" : file.slice(0, separator)) as SourceDirectory;
}

function artifactDirectoryFor(
  targetFile: `src/${string}`,
  artifact: "stories" | "tests",
): SourceDirectory {
  const directory = sourceDirectoryFor(targetFile);
  return directory.startsWith("src/components/") ? `${directory}/${artifact}` : directory;
}

export function primaryStoryTargetFor(entry: StoryTestbedPathTarget): `src/${string}` {
  return entry.targetFiles[0] ?? (`src/${entry.canonicalName}.ts` as `src/${string}`);
}

export function storyFileFor(
  canonicalName: string,
  targetFile: `src/${string}`,
): `src/${string}.art.vue` {
  return `${artifactDirectoryFor(targetFile, "stories")}/${canonicalName}.art.vue` as `src/${string}.art.vue`;
}

export function vueTestFileFor(
  canonicalName: string,
  targetFile: `src/${string}`,
): `src/${string}.vue.test.ts` {
  return `${artifactDirectoryFor(targetFile, "tests")}/${canonicalName}.vue.test.ts` as `src/${string}.vue.test.ts`;
}

export function browserTestFileFor(
  canonicalName: string,
  targetFile: `src/${string}`,
): `src/${string}.browser.spec.ts` {
  return `${artifactDirectoryFor(targetFile, "tests")}/${canonicalName}.browser.spec.ts` as `src/${string}.browser.spec.ts`;
}

export function vrtTestFileFor(
  canonicalName: string,
  targetFile: `src/${string}`,
): `src/${string}.vrt.spec.ts` {
  return `${artifactDirectoryFor(targetFile, "tests")}/${canonicalName}.vrt.spec.ts` as `src/${string}.vrt.spec.ts`;
}
