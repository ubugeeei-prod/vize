export const opinionatedSourceRoots = [
  "npm/",
  "docs/theme/",
  "playground/src/",
  "examples/vite-musea/src/",
  "examples/html-conformance/src/",
];

// Fixture sources intentionally violate rules; ordinary test code stays in scope.
const excludedDirectories = new Set(["__fixtures__", "__snapshots__", "fixtures"]);
const sourceExtension = /\.(?:vue|html?|[cm]?[jt]sx?)$/;
const fixtureFile = /\.fixture\.(?:vue|html?|[cm]?[jt]sx?)$/;

export function isOpinionatedSourceInput(file: string): boolean {
  return (
    opinionatedSourceRoots.some((root) => file.startsWith(root)) &&
    sourceExtension.test(file) &&
    !fixtureFile.test(file) &&
    !file.split("/").some((part) => excludedDirectories.has(part))
  );
}
