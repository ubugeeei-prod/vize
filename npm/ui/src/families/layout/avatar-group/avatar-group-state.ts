import { defaultAvatarGroupMessages } from "./avatar-group-types.ts";
import type {
  AvatarGroupMessageOverrides,
  AvatarGroupMessages,
  AvatarGroupState,
} from "./avatar-group-types.ts";

/** Result of {@link splitAvatarGroup}. */
export interface AvatarGroupSplit<Item> {
  readonly visibleItems: readonly Item[];
  readonly hiddenItems: readonly Item[];
  readonly overflowCount: number;
  readonly state: AvatarGroupState;
}

/**
 * Split items into visible tiles and an overflow tile.
 *
 * `max` counts every tile, including the overflow tile, so a group never
 * renders more than `max` tiles. `total` declares people beyond `items` (for
 * example a server-side count); they only ever add to the overflow count.
 * Invalid `max` values (non-finite, below 1) disable collapsing.
 */
export function splitAvatarGroup<Item>(
  items: readonly Item[],
  max?: number,
  total?: number,
): AvatarGroupSplit<Item> {
  const declared =
    total !== undefined && Number.isFinite(total)
      ? Math.max(items.length, Math.floor(total))
      : items.length;
  if (declared === 0) {
    return { visibleItems: [], hiddenItems: [], overflowCount: 0, state: "empty" };
  }
  const limit = max !== undefined && Number.isFinite(max) && max >= 1 ? Math.floor(max) : undefined;
  if (limit === undefined || declared <= limit) {
    const remainder = declared - items.length;
    if (remainder === 0) {
      return { visibleItems: items, hiddenItems: [], overflowCount: 0, state: "expanded" };
    }
    const visibleItems = limit === undefined ? items : items.slice(0, Math.max(0, limit - 1));
    const hiddenItems = items.slice(visibleItems.length);
    return {
      visibleItems,
      hiddenItems,
      overflowCount: declared - visibleItems.length,
      state: "collapsed",
    };
  }
  const visibleCount = Math.min(items.length, limit - 1);
  return {
    visibleItems: items.slice(0, visibleCount),
    hiddenItems: items.slice(visibleCount),
    overflowCount: declared - visibleCount,
    state: "collapsed",
  };
}

/** Merge partial message overrides over the English defaults. */
export function resolveAvatarGroupMessages(
  overrides: AvatarGroupMessageOverrides | undefined,
): AvatarGroupMessages {
  return { ...defaultAvatarGroupMessages, ...overrides };
}

/** Normalize a spacing value into a CSS length for `--vize-ui-avatar-group-spacing`. */
export function avatarGroupSpacing(spacing: number | string | undefined): string | undefined {
  if (spacing === undefined) return undefined;
  if (typeof spacing === "number") return Number.isFinite(spacing) ? `${spacing}px` : undefined;
  const trimmed = spacing.trim();
  return trimmed.length > 0 && !/[;{}]/.test(trimmed) ? trimmed : undefined;
}
