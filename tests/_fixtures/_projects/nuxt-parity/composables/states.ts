// Auto-imported as types via the `declare global` re-exports in
// `.nuxt/imports.d.ts`. `Box` is generic so a use site proves the re-export
// keeps the type's own parameter instead of collapsing to `any`.
export type ColorName = "pink" | "blue";

export interface Box<T> {
  value: T;
}
