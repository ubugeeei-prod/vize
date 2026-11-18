export {}
declare module 'vue' {
  interface ComponentCustomProperties {
    $t: (key: string) => string
    $filters: { money(n: number): string }
  }
}
