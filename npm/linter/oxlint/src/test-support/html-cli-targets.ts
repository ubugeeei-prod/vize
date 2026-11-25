/**
 * Several literal targets on the real pinned hosts: each one holding standalone
 * HTML gets its own native operation, joined like separate one-target runs.
 * These fixtures keep their own counter law, so the shared installed-package
 * campaign over `fixtures` stays unchanged.
 */
export interface TargetFixture {
  name: string;
  files: Record<string, string>;
  /** Authored bytes for inputs that only the host lints. */
  sources?: Record<string, string>;
  targets: string[];
  selected: Record<string, string>;
  /** The stock host lints these non-HTML files, so its report is not NoFilesFound. */
  plain?: boolean;
  nativeCalls: number;
  joinedCalls: number;
}

export const targetFixtures: TargetFixture[] = [
  {
    name: "html-target-beside-plain-target",
    files: { "src/Live.html": "Warning.html" },
    sources: { "scripts/build.ts": "export const ready = true;\n" },
    targets: ["src", "scripts"],
    selected: { "src/Live.html": "warning" },
    plain: true,
    nativeCalls: 1,
    joinedCalls: 1,
  },
  {
    name: "html-in-several-targets",
    files: { "src/Live.html": "Warning.html", "pages/Other.html": "Warning.html" },
    targets: ["pages", "src"],
    selected: { "pages/Other.html": "warning", "src/Live.html": "warning" },
    nativeCalls: 2,
    joinedCalls: 2,
  },
  {
    name: "html-file-inside-html-directory-target",
    files: { "src/Live.html": "Warning.html", "src/Clean.html": "Clean.html" },
    targets: ["src/Live.html", "src"],
    selected: { "src/Clean.html": "empty", "src/Live.html": "warning" },
    nativeCalls: 2,
    joinedCalls: 1,
  },
];

export const targetCounterLaw = { cases: 12, wholeNativeCalls: 20, wholeHostPhases: 12 };
