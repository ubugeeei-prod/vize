import { reactive, ref, type Ref } from "vue";

import type {
  NuxtMuseaNavigationTarget,
  NuxtMuseaOptions,
  NuxtMuseaRouteLocation,
  NuxtMuseaRouteQueryValue,
} from "./types.js";

const DEFAULT_REQUEST_URL = "http://localhost:3000/";

const routeState = reactive<NuxtMuseaRouteLocation>(createRoute({}));
const runtimeConfigState = reactive<Record<string, unknown>>({ public: {} });
const appConfigState = reactive<Record<string, unknown>>({});
const requestState = reactive<{ url: string; headers: Record<string, string> }>({
  url: DEFAULT_REQUEST_URL,
  headers: {},
});

const stateRefs = new Map<string, Ref<unknown>>();
const cookieRefs = new Map<string, Ref<unknown>>();
const callOnceKeys = new Set<string>();
const fetchMocksState: Record<string, unknown> = {};
const errorRef = ref<Error | null>(null);
let idCounter = 0;

export function configureNuxtMuseaMocks(options: NuxtMuseaOptions = {}): void {
  setRouteConfig(options.route ?? {});
  setRuntimeConfig(options.runtimeConfig ?? {});
  setAppConfig(options.appConfig ?? {});
  setFetchMocks(options.fetchMocks ?? {});
  setStateMocks(options.stateMocks ?? {});
  setCookieMocks(options.cookieMocks ?? {});
  setRequestConfig(options.request ?? {});
}

export function resetNuxtMuseaMocks(): void {
  setRouteConfig({});
  setRuntimeConfig({});
  setAppConfig({});
  setFetchMocks({});
  setRequestConfig({});
  stateRefs.clear();
  cookieRefs.clear();
  callOnceKeys.clear();
  errorRef.value = null;
  idCounter = 0;
}

export function getRouteState(): NuxtMuseaRouteLocation {
  return routeState;
}

export function setRouteConfig(config: Partial<NuxtMuseaRouteLocation>): void {
  Object.assign(routeState, createRoute(config));
}

export function resolveNavigationTarget(target: NuxtMuseaNavigationTarget): NuxtMuseaRouteLocation {
  if (typeof target === "string") {
    return createRoute({ path: target });
  }

  return createRoute({
    path: target.path,
    name: target.name ?? null,
    params: target.params,
    query: target.query,
    hash: target.hash,
    meta: typeof target.meta === "object" && target.meta != null ? target.meta : undefined,
  });
}

export function getRuntimeConfigState(): Record<string, unknown> {
  return runtimeConfigState;
}

export function setRuntimeConfig(config: NuxtMuseaOptions["runtimeConfig"]): void {
  replaceRecord(runtimeConfigState, { public: {}, ...config });
}

export function getAppConfigState(): Record<string, unknown> {
  return appConfigState;
}

export function setAppConfig(config: Record<string, unknown>): void {
  replaceRecord(appConfigState, config);
}

export function updateAppConfigState(config: Record<string, unknown>): void {
  Object.assign(appConfigState, config);
}

export function getFetchMocks(): Record<string, unknown> {
  return fetchMocksState;
}

export function setFetchMocks(mocks: Record<string, unknown>): void {
  replaceRecord(fetchMocksState, mocks);
}

export function getStateRef<T>(key: string, init?: () => T): Ref<T | undefined> {
  const existing = stateRefs.get(key);
  if (existing) {
    return existing as Ref<T | undefined>;
  }

  const value = init ? init() : undefined;
  const state = ref(value) as Ref<T | undefined>;
  stateRefs.set(key, state as Ref<unknown>);
  return state;
}

export function setStateMocks(mocks: Record<string, unknown>): void {
  stateRefs.clear();
  for (const [key, value] of Object.entries(mocks)) {
    stateRefs.set(key, ref(value));
  }
}

export function clearStateRefs(keys?: string | string[]): void {
  if (keys == null) {
    stateRefs.clear();
    return;
  }

  for (const key of Array.isArray(keys) ? keys : [keys]) {
    stateRefs.delete(key);
  }
}

export function getCookieRef<T>(name: string, init?: () => T): Ref<T | undefined> {
  const existing = cookieRefs.get(name);
  if (existing) {
    return existing as Ref<T | undefined>;
  }

  const value = init ? init() : undefined;
  const cookie = ref(value) as Ref<T | undefined>;
  cookieRefs.set(name, cookie as Ref<unknown>);
  return cookie;
}

export function setCookieMocks(mocks: Record<string, unknown>): void {
  cookieRefs.clear();
  for (const [key, value] of Object.entries(mocks)) {
    cookieRefs.set(key, ref(value));
  }
}

export function getRequestState(): { url: string; headers: Record<string, string> } {
  return requestState;
}

export function setRequestConfig(request: NonNullable<NuxtMuseaOptions["request"]>): void {
  requestState.url = request.url ?? DEFAULT_REQUEST_URL;
  requestState.headers = { ...request.headers };
}

export function getErrorRef(): Ref<Error | null> {
  return errorRef;
}

export function setError(error: Error | null): void {
  errorRef.value = error;
}

export function nextNuxtId(): string {
  idCounter += 1;
  return `musea-nuxt-${idCounter}`;
}

export async function runCallOnce<T>(
  key: string,
  fn: () => T | Promise<T>,
): Promise<T | undefined> {
  if (callOnceKeys.has(key)) {
    return undefined;
  }

  callOnceKeys.add(key);
  return await fn();
}

function createRoute(config: Partial<NuxtMuseaRouteLocation>): NuxtMuseaRouteLocation {
  const path = config.path ?? "/";
  const query = { ...config.query };
  const hash = normalizeHash(config.hash ?? "");
  return {
    path,
    name: config.name ?? inferRouteName(path),
    params: { ...config.params },
    query,
    hash,
    fullPath: config.fullPath ?? buildFullPath(path, query, hash),
    meta: { ...config.meta },
    matched: [...(config.matched ?? [])],
    redirectedFrom: config.redirectedFrom,
  };
}

function inferRouteName(path: string): string {
  const normalized = path.replace(/^\/+|\/+$/g, "");
  return normalized.length === 0 ? "index" : normalized.replace(/[^A-Za-z0-9_]+/g, "-");
}

function buildFullPath(
  path: string,
  query: Record<string, NuxtMuseaRouteQueryValue>,
  hash: string,
): string {
  const searchParams = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (value == null) continue;
    if (Array.isArray(value)) {
      for (const item of value) {
        searchParams.append(key, item);
      }
      continue;
    }
    searchParams.set(key, value);
  }

  const search = searchParams.toString();
  return `${path}${search ? `?${search}` : ""}${hash}`;
}

function normalizeHash(hash: string): string {
  if (!hash) return "";
  return hash.startsWith("#") ? hash : `#${hash}`;
}

function replaceRecord(target: Record<string, unknown>, value: Record<string, unknown>): void {
  for (const key of Object.keys(target)) {
    delete target[key];
  }
  Object.assign(target, value);
}
