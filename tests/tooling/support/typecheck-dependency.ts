type SkippableTest = { skip(reason: string): void };

function dependencyIsAvailable(value: unknown): boolean {
  return value !== null && value !== undefined && value !== false;
}

export function typecheckDependencySkip(
  value: unknown,
  label: string,
  skipReason: string,
  required = process.env.VIZE_TEST_REQUIRE_TSGO === "1",
): false | string {
  if (dependencyIsAvailable(value)) return false;

  if (required) {
    throw new Error(`${label} is required when VIZE_TEST_REQUIRE_TSGO=1`);
  }
  return skipReason;
}

export function requireTypecheckDependency<T>(
  t: SkippableTest,
  value: T | null | undefined | false,
  label: string,
  skipReason: string,
  required = process.env.VIZE_TEST_REQUIRE_TSGO === "1",
): T | undefined {
  const skip = typecheckDependencySkip(value, label, skipReason, required);
  if (skip !== false) {
    t.skip(skip);
    return undefined;
  }
  return value as T;
}
