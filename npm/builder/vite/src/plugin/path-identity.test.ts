import assert from "node:assert/strict";
import { test } from "node:test";
import { withoutVerbatimPrefix } from "./path-identity.ts";

void test("verbatim Windows paths take their plain spelling", () => {
  for (const [verbatim, plain] of [
    ["\\\\?\\C:\\app\\src\\types.ts", "C:\\app\\src\\types.ts"],
    ["\\\\?\\c:\\app\\tsconfig.json", "c:\\app\\tsconfig.json"],
    ["\\\\?\\C:\\", "C:\\"],
    ["\\\\?\\C:\\.vize\\a.b\\console.ts", "C:\\.vize\\a.b\\console.ts"],
    ["\\\\?\\UNC\\host\\share\\app\\types.ts", "\\\\host\\share\\app\\types.ts"],
    ["\\\\?\\unc\\host\\share", "\\\\host\\share"],
    ["//?/C:/app/src/types.ts", "C:/app/src/types.ts"],
    ["//?/UNC/host/share/app", "//host/share/app"],
    [`\\\\?\\C:\\${"node_modules\\".repeat(30)}`, `C:\\${"node_modules\\".repeat(30)}`],
  ]) {
    assert.equal(withoutVerbatimPrefix(verbatim), plain);
  }
});

void test("paths without a plain Windows spelling are kept", () => {
  for (const file of [
    "C:\\app\\types.ts",
    "\\\\host\\share\\types.ts",
    "/app/src/types.ts",
    "\\\\?\\C:",
    "\\\\?\\C:app",
    "\\\\?\\1:\\app",
    "\\\\?\\Volume{01234567-89ab-cdef-0123-456789abcdef}\\app",
    "\\\\?\\GLOBALROOT\\Device\\HarddiskVolume1\\app",
    "\\\\?\\UNC\\host",
    "\\\\?\\UNC\\\\share\\app",
    "\\\\?\\C:\\app\\..\\secret",
    "\\\\?\\C:\\app\\.\\file.ts",
    "\\\\?\\C:\\app\\trailing.",
    "\\\\?\\C:\\app\\trailing ",
    "\\\\?\\C:\\app\\con",
    "\\\\?\\C:\\app\\NUL.d.ts",
    "\\\\?\\C:\\app\\lpt¹.txt",
    "\\\\?\\C:\\app\\aux .ts",
    "\\\\?\\C:\\app\\a:b",
    "\\\\?\\C:\\app\\a/b",
    "\\\\?\\C:\\app\\a*b",
    `\\\\?\\C:\\${"a".repeat(256)}`,
    "\\\\.\\C:\\app",
  ]) {
    assert.equal(withoutVerbatimPrefix(file), file);
  }
});
