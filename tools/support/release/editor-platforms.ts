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

export function editorPlatformAssets() {
  return editorPlatformMatrix().flatMap(({ archive }) => [
    `editor-extension-artifacts/platform/${archive}`,
    `editor-extension-artifacts/platform/${archive}.receipt.json`,
  ]);
}

/**
 * The immutable tag decides which platform artifacts may exist. Platform
 * packaging never gates npm, so an incomplete cohort resumes without it.
 */
export function editorArtifactsForResume(workflow: string, artifactNames: string[]): boolean {
  const expected = editorPlatformMatrix().map(({ artifact }) => artifact);
  const actual = artifactNames.filter((name) => name.startsWith("editor-extension-platform-"));
  const allowed = /^  build-editor-platforms:\s*$/m.test(workflow) ? expected : [];
  if (new Set(actual).size !== actual.length || actual.some((name) => !allowed.includes(name))) {
    throw new Error("Immutable release editor platform artifact inventory is foreign");
  }
  return allowed.length > 0 && actual.length === allowed.length;
}
