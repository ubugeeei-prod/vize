import { describe, expect, it } from "vitest";
import * as path from "node:path";
import {
  createBoundedFileBatches,
  displayPath,
  sanitizeTerminalText,
  shouldPreferWorkspaceBinding,
  shouldRetainCheckSource,
} from "./cli";

describe("shouldPreferWorkspaceBinding", () => {
  it("detects the local workspace native package", () => {
    expect(
      shouldPreferWorkspaceBinding(
        `${path.sep}Users${path.sep}example${path.sep}repo${path.sep}npm${path.sep}vize-native${path.sep}index.js`,
      ),
    ).toBe(true);
  });

  it("ignores published platform packages", () => {
    expect(
      shouldPreferWorkspaceBinding(
        `${path.sep}repo${path.sep}node_modules${path.sep}.pnpm${path.sep}@vizejs+native-darwin-arm64${path.sep}node_modules${path.sep}@vizejs${path.sep}native-darwin-arm64${path.sep}index.js`,
      ),
    ).toBe(false);
  });

  it("returns false when the fallback package cannot be resolved", () => {
    expect(shouldPreferWorkspaceBinding(null)).toBe(false);
  });
});

describe("sanitizeTerminalText", () => {
  it("strips terminal escape and control sequences", () => {
    const sanitized = sanitizeTerminalText("bad\x1b]52;c;AAAA\x07\x1b[31mname");

    expect(sanitized).not.toContain("\x1b");
    expect(sanitized).not.toContain("\x07");
    expect(sanitized).toBe("badname");
  });

  it("keeps ordinary multiline terminal text readable", () => {
    expect(sanitizeTerminalText("line 1\n\tline 2\r\n")).toBe("line 1\n\tline 2\r\n");
  });

  it("sanitizes displayed path segments", () => {
    const unsafePath = path.join(process.cwd(), "bad\x1b[31m.vue");

    expect(displayPath(unsafePath)).toBe("bad.vue");
  });
});

describe("createBoundedFileBatches", () => {
  it("splits batches by file count and total source bytes", () => {
    const batches = createBoundedFileBatches(["a.vue", "b.vue", "c.vue", "d.vue"], {
      maxFiles: 3,
      maxBytes: 10,
      sizeOf(file) {
        return file === "c.vue" ? 9 : 4;
      },
    });

    expect(batches).toEqual([["a.vue", "b.vue"], ["c.vue"], ["d.vue"]]);
  });

  it("keeps a single oversized file processable", () => {
    const batches = createBoundedFileBatches(["huge.vue", "small.vue"], {
      maxFiles: 4,
      maxBytes: 10,
      sizeOf(file) {
        return file === "huge.vue" ? 100 : 1;
      },
    });

    expect(batches).toEqual([["huge.vue"], ["small.vue"]]);
  });
});

describe("shouldRetainCheckSource", () => {
  it("drops source retention for JSON and quiet check output", () => {
    expect(shouldRetainCheckSource({ format: "json" })).toBe(false);
    expect(shouldRetainCheckSource({ quiet: true })).toBe(false);
  });

  it("keeps source retention when diagnostics or declarations need it", () => {
    expect(shouldRetainCheckSource({})).toBe(true);
    expect(shouldRetainCheckSource({ format: "json", declaration: true })).toBe(true);
  });
});
