function parseSemver(version) {
  const match = /^(\d+)\.(\d+)\.(\d+)(?:[-+][0-9A-Za-z.-]+)?$/u.exec(version.trim());
  if (match === null) return null;
  return {
    major: Number.parseInt(match[1], 10),
    minor: Number.parseInt(match[2], 10),
    patch: Number.parseInt(match[3], 10),
  };
}

function compareSemver(left, right) {
  for (const key of ["major", "minor", "patch"]) {
    if (left[key] !== right[key]) return left[key] < right[key] ? -1 : 1;
  }
  return 0;
}

function satisfiesComparator(version, comparator) {
  const match = /^(>=|<=|>|<|=)?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$/u.exec(comparator);
  if (match === null) return false;
  const target = parseSemver(match[2]);
  if (target === null) return false;
  const comparison = compareSemver(version, target);
  switch (match[1] ?? "=") {
    case ">":
      return comparison > 0;
    case ">=":
      return comparison >= 0;
    case "<":
      return comparison < 0;
    case "<=":
      return comparison <= 0;
    case "=":
      return comparison === 0;
    default:
      return false;
  }
}

export function satisfiesVersionRange(version, range) {
  const parsedVersion = parseSemver(version);
  if (parsedVersion === null) return false;
  return range.split("||").some((rawRange) => {
    const current = rawRange.trim();
    if (current.startsWith("^")) {
      const base = parseSemver(current.slice(1));
      return (
        base !== null &&
        parsedVersion.major === base.major &&
        compareSemver(parsedVersion, base) >= 0
      );
    }
    if (current.startsWith("~")) {
      const base = parseSemver(current.slice(1));
      return (
        base !== null &&
        parsedVersion.major === base.major &&
        parsedVersion.minor === base.minor &&
        compareSemver(parsedVersion, base) >= 0
      );
    }
    const comparators = current.split(/\s+/u).filter(Boolean);
    return (
      comparators.length > 0 &&
      comparators.every((comparator) => satisfiesComparator(parsedVersion, comparator))
    );
  });
}
