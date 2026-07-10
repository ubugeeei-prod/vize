/**
 * Mock Nuxt navigation utilities.
 */

import { setRouteConfig, resolveNavigationTarget } from "../context.js";
import type { NuxtMuseaNavigationTarget } from "../types.js";

/**
 * Mock navigateTo - updates the shared route state in gallery context.
 */
export function navigateTo(
  to: NuxtMuseaNavigationTarget,
  _opts?: { replace?: boolean; redirectCode?: number; external?: boolean },
): Promise<void> {
  setRouteConfig(resolveNavigationTarget(to));
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

/**
 * Mock definePageMeta - page metadata is handled by Nuxt at build time.
 */
export function definePageMeta(_meta: Record<string, unknown>): void {
  // no-op
}

/**
 * Mock setPageLayout - no layout switching in isolated previews.
 */
export function setPageLayout(_layout: string | false): void {
  // no-op
}

export function prefetchComponents(_to: NuxtMuseaNavigationTarget): Promise<void> {
  return Promise.resolve();
}

export function preloadComponents(_to: NuxtMuseaNavigationTarget): Promise<void> {
  return Promise.resolve();
}

export function preloadRouteComponents(_to: NuxtMuseaNavigationTarget): Promise<void> {
  return Promise.resolve();
}
