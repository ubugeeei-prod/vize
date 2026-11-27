
import type { DefineComponent, SlotsType, VNodeProps } from 'vue'

type IslandSerialized<T> = T extends string | number ? T : { [K in keyof T]: IslandSerialized<T[K]> }
type IslandProps<T> = T extends new (...args: any) => { $props: infer P }
  ? new () => { $props: { [K in keyof P]: K extends keyof VNodeProps ? P[K] : P[K] & IslandSerialized<P[K]> } }
  : unknown
type IslandComponent<T> = DefineComponent<{}, { refresh: () => Promise<void> }, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, SlotsType<{ fallback: { error: unknown } }>> & T & IslandProps<T>

type HydrationStrategies = {
  hydrateOnVisible?: IntersectionObserverInit | true
  hydrateOnIdle?: number | true
  hydrateOnInteraction?: keyof HTMLElementEventMap | Array<keyof HTMLElementEventMap> | true
  hydrateOnMediaQuery?: string
  hydrateAfter?: number
  hydrateWhen?: boolean
  hydrateNever?: true
}
type LazyComponent<T> = DefineComponent<HydrationStrategies, {}, {}, {}, {}, {}, {}, { hydrated: () => void }> & T


export const TheGreeting: typeof import("../components/TheGreeting.vue")['default']
export const LazyTheGreeting: LazyComponent<typeof import("../components/TheGreeting.vue")['default']>

export const ServerProps: IslandComponent<typeof import("../components/ServerProps.server.vue")['default']>
export const LazyServerProps: LazyComponent<IslandComponent<typeof import("../components/ServerProps.server.vue")['default']>>

export const componentNames: string[]
