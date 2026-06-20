/**
 * Mock Nuxt navigation utilities.
 */

/**
 * Mock navigateTo - no-op in gallery context.
 */
export function navigateTo(
  _to: string | Record<string, unknown>,
  _opts?: { replace?: boolean; redirectCode?: number; external?: boolean },
): Promise<void> {
  return Promise.resolve();
}

/**
 * Mock abortNavigation - no-op in gallery context.
 */
export function abortNavigation(_err?: string | Error): void {
  // no-op
}

/**
 * Mock defineNuxtRouteMiddleware - returns the middleware function as-is.
 */
export function defineNuxtRouteMiddleware(middleware: unknown): unknown {
  return middleware;
}
