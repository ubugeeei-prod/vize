export function createFilter(
  include?: string | RegExp | (string | RegExp)[],
  exclude?: string | RegExp | (string | RegExp)[],
): (id: string) => boolean {
  const includePatterns = include ? (Array.isArray(include) ? include : [include]) : [/\.vue$/];
  const excludePatterns = exclude
    ? Array.isArray(exclude)
      ? exclude
      : [exclude]
    : [/node_modules/];

  return (id: string) => {
    const matchInclude = includePatterns.some((pattern) => matchesFilterPattern(pattern, id));
    const matchExclude = excludePatterns.some((pattern) => matchesFilterPattern(pattern, id));
    return matchInclude && !matchExclude;
  };
}

const GLOB_META = /[*?]/;

function matchesFilterPattern(pattern: string | RegExp, id: string): boolean {
  if (typeof pattern === "string") {
    return matchesStringFilter(pattern, id);
  }

  pattern.lastIndex = 0;
  const matched = pattern.test(id);
  pattern.lastIndex = 0;
  return matched;
}

function matchesStringFilter(pattern: string, id: string): boolean {
  const normalizedId = normalizeFilterPath(id);
  const normalizedPattern = normalizeFilterPath(pattern);
  if (!GLOB_META.test(normalizedPattern)) {
    return normalizedId.includes(normalizedPattern);
  }
  return stringGlobToRegExp(normalizedPattern).test(normalizedId);
}

function normalizeFilterPath(value: string): string {
  return value.replace(/\\/g, "/");
}

function stringGlobToRegExp(pattern: string): RegExp {
  const absolute = pattern.startsWith("/");
  let body = absolute ? pattern : stripRelativePrefix(pattern);
  let descendantSuffix = false;

  if (body.endsWith("/**")) {
    body = body.slice(0, -3);
    descendantSuffix = true;
  }

  const prefix = absolute ? "^" : "(?:^|/)";
  const suffix = descendantSuffix ? "(?:/.*)?$" : "$";
  return new RegExp(`${prefix}${globBodyToRegExp(body)}${suffix}`);
}

function stripRelativePrefix(pattern: string): string {
  let rest = pattern;
  while (rest.startsWith("../")) {
    rest = rest.slice(3);
  }
  while (rest.startsWith("./")) {
    rest = rest.slice(2);
  }
  return rest;
}

function globBodyToRegExp(pattern: string): string {
  let source = "";
  for (let i = 0; i < pattern.length; ) {
    const char = pattern[i];
    if (char === "*") {
      if (pattern[i + 1] === "*") {
        if (pattern[i + 2] === "/") {
          source += "(?:.*\\/)?";
          i += 3;
        } else {
          source += ".*";
          i += 2;
        }
      } else {
        source += "[^/]*";
        i += 1;
      }
      continue;
    }
    if (char === "?") {
      source += "[^/]";
      i += 1;
      continue;
    }
    source += escapeRegExp(char);
    i += 1;
  }
  return source;
}

function escapeRegExp(char: string): string {
  return "\\^$+?.()|[]{}".includes(char) ? `\\${char}` : char;
}
