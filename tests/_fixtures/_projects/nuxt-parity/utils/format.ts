// Reached through the `~/utils/format` path alias declared in
// `.nuxt/tsconfig.json`. The strict signature lets a wrong-typed call surface.
export function shout(label: string): string {
  return `${label.toUpperCase()}!`;
}
