// The publisher target is distinct from the Rust producer target. Keep this
// closed inventory shared by packaging, registry publication and signing.
export const editorReleasePlatforms = [
  { rustTarget: "x86_64-apple-darwin", target: "darwin-x64" },
  { rustTarget: "aarch64-apple-darwin", target: "darwin-arm64" },
  { rustTarget: "x86_64-pc-windows-msvc", target: "win32-x64" },
  { rustTarget: "aarch64-pc-windows-msvc", target: "win32-arm64" },
  { rustTarget: "x86_64-unknown-linux-gnu", target: "linux-x64" },
  { rustTarget: "aarch64-unknown-linux-gnu", target: "linux-arm64" },
  { rustTarget: "x86_64-unknown-linux-musl", target: "alpine-x64" },
  { rustTarget: "aarch64-unknown-linux-musl", target: "alpine-arm64" },
] as const;

export function editorPlatformMatrix() {
  return editorReleasePlatforms.map(({ rustTarget, target }) => ({
    rustTarget,
    target,
    archive: `vize-${target}.vsix`,
    artifact: `editor-extension-platform-${target}`,
    sourcePath: `vize-${target}.vsix`,
  }));
}

export function editorPublishMatrix() {
  return [
    {
      rustTarget: "",
      target: "universal",
      archive: "vize.vsix",
      artifact: "editor-extensions",
      sourcePath: "editors/vscode/dist/vize.vsix",
    },
    ...editorPlatformMatrix(),
  ];
}

export function editorPlatformAssets() {
  return editorPlatformMatrix().flatMap(({ archive }) => [
    `editor-extension-artifacts/platform/${archive}`,
    `editor-extension-artifacts/platform/${archive}.receipt.json`,
  ]);
}

/** The immutable tag decides the artifact cohort; absence is not a fallback. */
export function editorArtifactsForResume(workflow: string, artifactNames: string[]): boolean {
  const expected = editorPlatformMatrix().map(({ artifact }) => artifact);
  const actual = artifactNames.filter((name) => name.startsWith("editor-extension-platform-"));
  const required = /^  build-editor-platforms:\s*$/m.test(workflow);
  if (JSON.stringify(actual.toSorted()) !== JSON.stringify((required ? expected : []).toSorted())) {
    throw new Error(
      "Immutable release editor platform artifact inventory is incomplete or foreign",
    );
  }
  return required;
}
