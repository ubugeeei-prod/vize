// Browser-free form of the release review's link predicate
// (navigation-render-page.ts "links" phase), for the merge queue.
import { readFileSync, statSync } from "node:fs";
import path from "node:path";

const origin = "http://docs.invalid";

const entities: Record<string, string> = { amp: "&", lt: "<", gt: ">", quot: '"', apos: "'" };

export function decodeAttribute(value: string): string {
  return value.replace(/&(#x[0-9a-f]+|#[0-9]+|[a-z]+);/gi, (match, entity: string) => {
    if (entity[0] === "#") {
      const code =
        entity[1] === "x" || entity[1] === "X"
          ? Number.parseInt(entity.slice(2), 16)
          : Number.parseInt(entity.slice(1), 10);
      return String.fromCodePoint(code);
    }
    return entities[entity.toLowerCase()] ?? match;
  });
}

/** The review reads `.content a[href], .feature-card[href]`. */
export function reviewedHrefs(html: string): string[] {
  const hrefs: string[] = [];
  const start = html.search(/<article\b[^>]*\bclass="content"/);
  if (start !== -1) {
    const tags = /<(\/?)article\b[^>]*>/g;
    tags.lastIndex = start;
    let depth = 0;
    let end = html.length;
    for (let tag = tags.exec(html); tag; tag = tags.exec(html)) {
      depth += tag[1] ? -1 : 1;
      if (depth === 0) {
        end = tags.lastIndex;
        break;
      }
    }
    for (const anchor of html.slice(start, end).matchAll(/<a\b[^>]*?\shref="([^"]*)"/g)) {
      hrefs.push(decodeAttribute(anchor[1]!));
    }
  }
  for (const card of html.matchAll(
    /<[a-z]+\b(?=[^>]*\bclass="[^"]*\bfeature-card\b)[^>]*?\shref="([^"]*)"/g,
  )) {
    hrefs.push(decodeAttribute(card[1]!));
  }
  return hrefs;
}

/** Mirrors verify-navigation-render.ts's static server. */
function readServed(dist: string, pathname: string): string | undefined {
  let file = path.resolve(dist, `.${decodeURIComponent(pathname)}`);
  if (!file.startsWith(`${dist}${path.sep}`) && file !== dist) return undefined;
  try {
    if (statSync(file).isDirectory()) file = path.join(file, "index.html");
    return readFileSync(file, "utf8");
  } catch {
    return undefined;
  }
}

/** Broken same-site links and anchors on the given routes of a built site. */
export function brokenLinks(dist: string, routes: readonly string[]): string[] {
  const root = path.resolve(dist);
  const failures: string[] = [];
  const pages = new Map<string, string | undefined>();
  const served = (pathname: string) => {
    if (!pages.has(pathname)) pages.set(pathname, readServed(root, pathname));
    return pages.get(pathname);
  };
  for (const route of routes) {
    // navigation-render-page.ts opens the same URL.
    const base = `${origin}${route.replace(/\/$/, "")}/index.html`;
    const html = served(new URL(base).pathname);
    if (html === undefined) {
      failures.push(`${route}: page was not built`);
      continue;
    }
    for (const raw of new Set(reviewedHrefs(html))) {
      let target: URL;
      try {
        target = new URL(raw, base);
      } catch {
        continue;
      }
      if (target.origin !== origin) continue;
      const linked = served(target.pathname);
      if (linked === undefined) {
        failures.push(`${route}: broken link ${raw}`);
        continue;
      }
      if (target.hash) {
        const id = decodeURIComponent(target.hash.slice(1));
        if (!linked.includes(`id="${id}"`) && !linked.includes(`name="${id}"`)) {
          failures.push(`${route}: missing anchor ${raw}`);
        }
      }
    }
  }
  return failures;
}
