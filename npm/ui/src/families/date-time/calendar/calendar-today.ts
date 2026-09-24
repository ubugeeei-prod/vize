import { computed, onMounted, shallowRef } from "vue";
import type { ComputedRef } from "vue";

import { localeContext } from "../../i18n/locale/locale-runtime.ts";
import { fromEpochMilliseconds, normalizePlainDate } from "./plain-date.ts";
import type { PlainDate } from "./plain-date.ts";

/** Clock source accepted by date-time components: epoch milliseconds or a `Date`. */
export type DateTimeNow = () => number | Date;

/** Reactive inputs for {@link useToday}. */
export interface TodayOptions {
  /** Explicit current date. Wins over every clock. */
  readonly today: () => PlainDate | null | undefined;

  /** Injectable clock; evaluated during setup on the server and the client. */
  readonly now: () => DateTimeNow | undefined;

  /** IANA time zone used to turn clock instants into a calendar date. */
  readonly timeZone: () => string | null | undefined;
}

function readClock(now: DateTimeNow): number {
  const value = now();
  return value instanceof Date ? value.getTime() : value;
}

function hostTimeZone(): string {
  try {
    return new Intl.DateTimeFormat().resolvedOptions().timeZone;
  } catch {
    return "UTC";
  }
}

/**
 * Resolve "today" without making server output depend on the wall clock.
 *
 * Resolution order:
 * 1. `today` — an explicit date, deterministic everywhere.
 * 2. `now` — an injected clock (for example the request timestamp), read in
 *    `timeZone`, else the nearest `LocaleProvider` time zone, else `UTC`.
 *    Server and client must inject the same instant to hydrate cleanly.
 * 3. Otherwise `null` during SSR and hydration. After mount the host clock is
 *    read once in `timeZone`, the provided locale time zone, or the host zone.
 *
 * Must be called during component setup.
 */
export function useToday(options: TodayOptions): ComputedRef<PlainDate | null> {
  const locale = localeContext.useOptional();
  const mountedToday = shallowRef<PlainDate | null>(null);
  const zone = () => options.timeZone()?.trim() || locale?.timeZone;
  const injected = computed(() => {
    const explicit = normalizePlainDate(options.today());
    if (explicit) return explicit;
    const now = options.now();
    return now ? fromEpochMilliseconds(readClock(now), zone() ?? "UTC") : null;
  });

  onMounted(() => {
    if (injected.value) return;
    mountedToday.value = fromEpochMilliseconds(Date.now(), zone() ?? hostTimeZone());
  });

  return computed(() => injected.value ?? mountedToday.value);
}
