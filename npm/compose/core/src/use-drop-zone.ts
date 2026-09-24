import { readonly, ref, shallowRef, toValue, unref, watch } from "vue";
import type { MaybeRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** One entry of `DataTransfer.items`. */
export interface DataTransferItemLike {
  /** `"file"` or `"string"`. */
  readonly kind: string;

  /** MIME type of the item. */
  readonly type: string;
}

/** Minimal `DataTransfer` consumed by {@link useDropZone}. */
export interface DataTransferLike {
  /** Items being dragged; types are readable during the drag. */
  readonly items?: ArrayLike<DataTransferItemLike>;

  /** Dropped files; populated on `drop`. */
  readonly files: ArrayLike<File>;

  /** Feedback shown to the user (`"copy"`, `"none"`, …). */
  dropEffect: string;
}

/** File name and MIME type used for acceptance checks. */
export interface DropZoneFileInfo {
  /** File name including extension. Empty while dragging. */
  readonly name: string;

  /** MIME type, possibly empty for unknown types. */
  readonly type: string;
}

/**
 * Accepted files: `accept`-attribute style patterns (`"image/*"`,
 * `"application/pdf"`, `".png"`), or a predicate over the dragged MIME types.
 */
export type DropZoneAccept = readonly string[] | ((types: readonly string[]) => boolean);

/** Options for {@link useDropZone}. */
export interface UseDropZoneOptions {
  /**
   * Accepted files. Extension patterns can only be verified on drop
   * because browsers hide file names while dragging. A ref rather than a
   * getter, because a predicate is itself a function.
   *
   * @default every file
   */
  readonly accept?: MaybeRef<DropZoneAccept | undefined>;

  /**
   * Accept more than one file per drop.
   *
   * @default true
   */
  readonly multiple?: MaybeRefOrGetter<boolean>;

  /**
   * Call `preventDefault` even for rejected drags, so the browser never
   * opens a rejected file dropped onto the zone.
   *
   * @default false
   */
  readonly preventDefaultForUnhandled?: boolean;

  /**
   * Called with the accepted files (or `null` when none were accepted).
   *
   * @default undefined
   */
  readonly onDrop?: (files: readonly File[] | null, event: Event) => void;

  /**
   * Called when a drag enters the zone.
   *
   * @default undefined
   */
  readonly onEnter?: (event: Event) => void;

  /**
   * Called when a drag leaves the zone (including its children).
   *
   * @default undefined
   */
  readonly onLeave?: (event: Event) => void;

  /**
   * Called for every `dragover` inside the zone.
   *
   * @default undefined
   */
  readonly onOver?: (event: Event) => void;
}

/** Reactive state returned by {@link useDropZone}. */
export interface DropZoneControls {
  /** Whether a drag is currently over the zone (children included). */
  readonly isOverDropZone: Readonly<Ref<boolean>>;

  /** Whether the current drag is acceptable. */
  readonly accepted: Readonly<Ref<boolean>>;

  /** Files accepted by the most recent drop. */
  readonly files: Readonly<ShallowRef<readonly File[] | null>>;
}

function matchesPattern(pattern: string, file: DropZoneFileInfo): boolean {
  const normalized = pattern.trim().toLowerCase();
  if (normalized === "" || normalized === "*" || normalized === "*/*") return true;
  if (normalized.startsWith(".")) {
    // While dragging the name is unknown: treat extensions as provisionally
    // acceptable and verify them on drop.
    return file.name === "" || file.name.toLowerCase().endsWith(normalized);
  }
  const type = file.type.toLowerCase();
  if (normalized.endsWith("/*")) return type.startsWith(normalized.slice(0, -1));
  return type === normalized;
}

/**
 * Test a file against `accept`-attribute style patterns.
 *
 * Supports exact MIME types, wildcard subtypes (`"image/*"`), extensions
 * (`".png"`, case-insensitive), and `"*"`. An empty pattern list accepts
 * everything. An empty `name` (the file is still being dragged) makes
 * extension patterns match provisionally.
 *
 * @param accept Patterns to test.
 * @param file File name and MIME type.
 * @returns Whether any pattern matches.
 */
export function matchesAccept(accept: readonly string[], file: DropZoneFileInfo): boolean {
  return accept.length === 0 || accept.some((pattern) => matchesPattern(pattern, file));
}

function dataTransferOf(event: Event): DataTransferLike | undefined {
  if (!("dataTransfer" in event)) return undefined;
  const transfer: unknown = event.dataTransfer;
  if (typeof transfer !== "object" || transfer === null || !("files" in transfer)) return undefined;
  return isDataTransfer(transfer) ? transfer : undefined;
}

function isDataTransfer(candidate: object): candidate is DataTransferLike {
  return "files" in candidate && "dropEffect" in candidate;
}

function draggedTypes(transfer: DataTransferLike): readonly string[] {
  const types: string[] = [];
  const items = transfer.items;
  if (!items) return types;
  for (let index = 0; index < items.length; index += 1) {
    const item = items[index];
    if (item?.kind === "file") types.push(item.type);
  }
  return types;
}

/**
 * Headless drop-zone logic for any element.
 *
 * Tracks whether a drag is over the target (a nesting counter keeps
 * `dragleave` from child elements from flickering the state), decides
 * acceptance from the dragged MIME types while dragging, and filters the
 * dropped files by `accept` (including extension patterns) and `multiple`.
 * Rejected drags show the `"none"` drop effect. Rendering is entirely up to
 * the caller. Listeners follow the reactive target and are removed when the
 * owning reactive scope stops.
 *
 * Server rendering: no listener is attached; all state is empty/false.
 *
 * @example
 * ```ts
 * const zone = useTemplateRef<HTMLElement>("zone");
 * const { isOverDropZone, files } = useDropZone(zone, { accept: ["image/*"] });
 * ```
 *
 * @param target Reactive drop target.
 * @param options Acceptance rules and lifecycle callbacks.
 * @default options {}
 * @returns Drag and drop state.
 */
export function useDropZone(
  target: MaybeRefOrGetter<EventTarget | null | undefined>,
  options: UseDropZoneOptions = {},
): DropZoneControls {
  const isOverDropZone = ref(false);
  const accepted = ref(false);
  const files = shallowRef<readonly File[] | null>(null);
  let depth = 0;

  const acceptsTypes = (types: readonly string[]): boolean => {
    const accept = unref(options.accept);
    if (types.length === 0) return false;
    if (!(toValue(options.multiple) ?? true) && types.length > 1) return false;
    if (accept === undefined) return true;
    if (typeof accept === "function") return accept(types);
    return types.every((type) => matchesAccept(accept, { name: "", type }));
  };

  const acceptedFiles = (transfer: DataTransferLike): readonly File[] => {
    const dropped = Array.from(transfer.files);
    const accept = unref(options.accept);
    const matching =
      accept === undefined
        ? dropped
        : typeof accept === "function"
          ? accept(dropped.map((file) => file.type))
            ? dropped
            : []
          : dropped.filter((file) => matchesAccept(accept, file));
    if (matching.length !== dropped.length) return [];
    return !(toValue(options.multiple) ?? true) && matching.length > 1 ? [] : matching;
  };

  const settleDefault = (event: Event, transfer: DataTransferLike | undefined): void => {
    if (accepted.value || (options.preventDefaultForUnhandled ?? false)) event.preventDefault();
    if (transfer) transfer.dropEffect = accepted.value ? "copy" : "none";
  };

  const onEnter = (event: Event): void => {
    const transfer = dataTransferOf(event);
    depth += 1;
    if (depth === 1) {
      accepted.value = transfer !== undefined && acceptsTypes(draggedTypes(transfer));
      isOverDropZone.value = true;
      options.onEnter?.(event);
    }
    settleDefault(event, transfer);
  };

  const onOver = (event: Event): void => {
    settleDefault(event, dataTransferOf(event));
    options.onOver?.(event);
  };

  const onLeave = (event: Event): void => {
    depth = Math.max(0, depth - 1);
    if (depth > 0) return;
    isOverDropZone.value = false;
    accepted.value = false;
    options.onLeave?.(event);
  };

  const onDrop = (event: Event): void => {
    const transfer = dataTransferOf(event);
    const dropped = transfer ? acceptedFiles(transfer) : [];
    if (dropped.length > 0 || (options.preventDefaultForUnhandled ?? false)) {
      event.preventDefault();
    }
    depth = 0;
    isOverDropZone.value = false;
    accepted.value = false;
    files.value = dropped.length > 0 ? dropped : null;
    options.onDrop?.(files.value, event);
  };

  const stop = watch(
    () => toValue(target) ?? undefined,
    (element, _previous, onCleanup) => {
      depth = 0;
      isOverDropZone.value = false;
      accepted.value = false;
      if (!element) return;
      element.addEventListener("dragenter", onEnter);
      element.addEventListener("dragover", onOver);
      element.addEventListener("dragleave", onLeave);
      element.addEventListener("drop", onDrop);
      onCleanup(() => {
        element.removeEventListener("dragenter", onEnter);
        element.removeEventListener("dragover", onOver);
        element.removeEventListener("dragleave", onLeave);
        element.removeEventListener("drop", onDrop);
      });
    },
    { immediate: true, flush: "sync" },
  );

  tryOnScopeDispose(stop);

  return {
    isOverDropZone: readonly(isOverDropZone),
    accepted: readonly(accepted),
    files,
  };
}
