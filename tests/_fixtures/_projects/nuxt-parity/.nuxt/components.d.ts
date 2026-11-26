
import type { DefineComponent } from 'vue'

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

export const componentNames: string[]
