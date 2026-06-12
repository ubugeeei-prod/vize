export function createFilter(
  include?: string | RegExp | Array<string | RegExp>,
  exclude?: string | RegExp | Array<string | RegExp>,
): (id: string) => boolean {
  const includePatterns = include
    ? Array.isArray(include)
      ? include
      : [include]
    : [/\.vue$/, /\.[jt]sx$/];
  const excludePatterns = exclude
    ? Array.isArray(exclude)
      ? exclude
      : [exclude]
    : [/node_modules/];

  return (id: string) => {
    const matchInclude = includePatterns.some((pattern) => matchesPattern(pattern, id));
    const matchExclude = excludePatterns.some((pattern) => matchesPattern(pattern, id));
    return matchInclude && !matchExclude;
  };
}

function matchesPattern(pattern: string | RegExp, id: string): boolean {
  if (typeof pattern === "string") {
    return id.includes(pattern);
  }

  pattern.lastIndex = 0;
  const matches = pattern.test(id);
  pattern.lastIndex = 0;
  return matches;
}
