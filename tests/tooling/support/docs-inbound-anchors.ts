// Fragment links between documentation pages, checked against the natively rendered heading ids.
import { readFileSync, readdirSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";

const root = resolve(import.meta.dirname, "../../..");
const content = join(root, "docs/content");
const require = createRequire(join(root, "docs/package.json"));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");

const walk = (directory: string): string[] =>
  readdirSync(directory).flatMap((name) => {
    const path = join(directory, name);
    if (statSync(path).isDirectory()) return walk(path);
    return path.endsWith(".md") ? [path] : [];
  });
let pages: string[] | undefined;

/** Fragments that Markdown links anywhere in `docs/content` point at in `target`. */
export function inboundFragments(target: string): Set<string> {
  pages ??= walk(content);
  const absolute = join(content, target);
  const fragments = new Set<string>();
  for (const page of pages) {
    const source = readFileSync(page, "utf8").replace(/^```[\s\S]*?^```/gm, "");
    for (const [, path, fragment] of source.matchAll(/\]\(([^)\s#]*)#([^)\s]+)\)/g)) {
      if (/^(?:[a-z][a-z+.-]*:|\/\/)/.test(path)) continue;
      const file = !path
        ? page
        : path.startsWith("/")
          ? join(content, path)
          : resolve(dirname(page), path);
      if (file === absolute || `${file}.md` === absolute)
        fragments.add(decodeURIComponent(fragment));
    }
  }
  return fragments;
}

/** Element ids in the native render of `target`. */
export function renderedIds(target: string): Set<string> {
  const result = native.transform(readFileSync(join(content, target), "utf8"), {});
  if (result.errors.length > 0) throw new Error(`${target}: ${result.errors.join(", ")}`);
  return new Set(Array.from(result.html.matchAll(/\bid="([^"]+)"/g), (match) => match[1]));
}

/** Inbound fragments of `target` that no rendered element carries. */
export function missingInboundAnchors(target: string): string[] {
  const ids = renderedIds(target);
  return [...inboundFragments(target)].filter((fragment) => !ids.has(fragment));
}
