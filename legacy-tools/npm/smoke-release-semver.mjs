import semver from "semver";

export function satisfiesVersionRange(version, range) {
  return semver.satisfies(version, range);
}
