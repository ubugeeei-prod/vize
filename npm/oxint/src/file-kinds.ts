export function isVueLikeFile(filename: string): boolean {
  return hasVueLikeExtension(filename);
}

export function hasVueLikeExtension(filename: string): boolean {
  return filename.endsWith(".vue") || filename.endsWith(".html") || filename.endsWith(".htm");
}

export function isStandaloneHtmlFile(filename: string): boolean {
  return filename.endsWith(".html") || filename.endsWith(".htm");
}
