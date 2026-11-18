import type { InjectionKey } from 'vue'
export const themeKey: InjectionKey<{ dark: boolean }> = Symbol('theme')
