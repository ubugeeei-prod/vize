export function isVueLikeFile(filename: string): boolean {
  return hasVueLikeExtension(filename);
}

export function isPatinaFile(filename: string): boolean {
  return isVueLikeFile(filename) || isScriptLikeFile(filename);
}

export function isScriptLikeFile(filename: string): boolean {
  return /\.(?:[cm]?[jt]sx?)$/u.test(filename);
}

/** The script files the native linter lints as plain modules, without JSX. */
export function isPlainScriptFile(filename: string): boolean {
  return /\.(?:[cm]?[jt]s)$/u.test(filename);
}

export function hasVueLikeExtension(filename: string): boolean {
  return filename.endsWith(".vue") || filename.endsWith(".html") || filename.endsWith(".htm");
}

export function isStandaloneHtmlFile(filename: string): boolean {
  return filename.endsWith(".html") || filename.endsWith(".htm");
}
