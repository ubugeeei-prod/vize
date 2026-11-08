export * from "./index.ts";

/** A documented public option contract. */
export interface FixtureOptions {
  /** @default false */
  readonly enabled?: boolean;
}

/** A documented public value. */
export const fixture = true;
