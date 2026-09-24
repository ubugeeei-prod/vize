import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** ARIA landmark roles rendered or discovered by the Landmark family. */
export type LandmarkRole =
  | "banner"
  | "complementary"
  | "contentinfo"
  | "form"
  | "main"
  | "navigation"
  | "region"
  | "search";

/**
 * Landmark roles that may repeat on a page and therefore need an accessible
 * name to be distinguishable (`region` and `form` are only exposed as
 * landmarks when named).
 */
export type NamedLandmarkRole = "complementary" | "form" | "navigation" | "region" | "search";

/** Native element rendered for each landmark role. */
export interface LandmarkElementMap {
  readonly banner: "header";
  readonly complementary: "aside";
  readonly contentinfo: "footer";
  readonly form: "form";
  readonly main: "main";
  readonly navigation: "nav";
  readonly region: "section";
  readonly search: "search";
}

/** One landmark known to a navigation controller, in document order. */
export interface LandmarkInfo {
  /** Element id, or `null` for discovered landmarks without one. */
  readonly id: string | null;

  /** Landmark role. */
  readonly role: LandmarkRole;

  /** Resolved accessible name, or `null` when unnamed. */
  readonly label: string | null;

  /** Landmark element. */
  readonly element: HTMLElement;
}

/** Key and modifier combination that triggers landmark cycling. */
export interface LandmarkKeyBinding {
  /** `KeyboardEvent.key` value, for example `"F6"`. */
  readonly key: string;

  /**
   * Required Shift state.
   *
   * @default false
   */
  readonly shiftKey?: boolean;

  /**
   * Required Alt/Option state.
   *
   * @default false
   */
  readonly altKey?: boolean;

  /**
   * Required Control state.
   *
   * @default false
   */
  readonly ctrlKey?: boolean;

  /**
   * Required Meta/Command state.
   *
   * @default false
   */
  readonly metaKey?: boolean;
}

/** Options accepted by {@link createLandmarkNavigation} and {@link useLandmarkNavigation}. */
export interface LandmarkNavigationOptions {
  /**
   * Element whose subtree is searched in discovery mode and whose document
   * receives the keyboard listener. `null` uses the document body.
   *
   * @default null
   */
  readonly root?: MaybeRefOrGetter<HTMLElement | null | undefined>;

  /**
   * Also cycle through native and `[role]` landmarks that were not rendered
   * by `Landmark`.
   *
   * @default false
   */
  readonly discover?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Whether keyboard cycling is active.
   *
   * @default true
   */
  readonly enabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Key that moves to the next landmark. `null` disables it; `undefined` uses F6.
   *
   * @default { key: "F6" }
   */
  readonly nextKey?: MaybeRefOrGetter<LandmarkKeyBinding | null | undefined>;

  /**
   * Key that moves to the previous landmark. `null` disables it; `undefined` uses Shift+F6.
   *
   * @default { key: "F6", shiftKey: true }
   */
  readonly previousKey?: MaybeRefOrGetter<LandmarkKeyBinding | null | undefined>;

  /** Called after focus moves to a landmark. */
  readonly onNavigate?: (landmark: LandmarkInfo, event: KeyboardEvent | null) => void;
}

/** Input accepted by {@link LandmarkNavigationController.register}. */
export interface LandmarkRegistrationInput {
  /** Stable landmark id. */
  readonly id: string;

  /** Landmark role. */
  readonly role: LandmarkRole;

  /** Rendered element, `null` before mount. */
  readonly element: Readonly<ShallowRef<HTMLElement | null>>;
}

/** Keyboard landmark navigation over registered and discovered landmarks. */
export interface LandmarkNavigationController {
  /** Landmarks in document order, refreshed on registration changes and before navigation. */
  readonly landmarks: Readonly<ShallowRef<readonly LandmarkInfo[]>>;

  /** Register one rendered landmark and return its unregister callback. */
  readonly register: (input: LandmarkRegistrationInput) => () => void;

  /** Re-read registered and discovered landmarks. */
  readonly refresh: () => readonly LandmarkInfo[];

  /** Focus the landmark after the one containing focus, wrapping at the end. */
  readonly focusNext: (event?: KeyboardEvent | null) => LandmarkInfo | null;

  /** Focus the landmark before the one containing focus, wrapping at the start. */
  readonly focusPrevious: (event?: KeyboardEvent | null) => LandmarkInfo | null;

  /** Focus the first landmark whose id or role matches. */
  readonly focusLandmark: (idOrRole: string) => LandmarkInfo | null;

  /** Keydown handler that cycles on the configured keys. */
  readonly handleKeydown: (event: KeyboardEvent) => void;

  /** Attach the keydown listener to the root's document. Idempotent. */
  readonly attach: () => void;

  /** Remove the keydown listener. */
  readonly detach: () => void;

  /** Detach and release registrations. */
  readonly dispose: () => void;
}

/** State exposed to Landmark slots. */
export interface LandmarkSlotState {
  /** Landmark role. */
  readonly role: LandmarkRole;

  /** Whether the landmark currently holds focus from landmark cycling. */
  readonly focused: boolean;
}

/** Public instance exposed by Landmark. */
export interface LandmarkExpose {
  /** Resolved landmark id. */
  readonly id: string;

  /** Rendered landmark element. */
  readonly element: HTMLElement | null;

  /** Focus the landmark element, making it programmatically focusable when needed. */
  readonly focus: (options?: FocusOptions) => boolean;
}

/** Public instance exposed by LandmarkProvider. */
export interface LandmarkProviderExpose {
  /** Landmarks in document order. */
  readonly landmarks: readonly LandmarkInfo[];

  /** Focus the next landmark. */
  readonly focusNext: () => LandmarkInfo | null;

  /** Focus the previous landmark. */
  readonly focusPrevious: () => LandmarkInfo | null;

  /** Focus a landmark by id or role. */
  readonly focusLandmark: (idOrRole: string) => LandmarkInfo | null;

  /** Re-read landmarks. */
  readonly refresh: () => readonly LandmarkInfo[];
}
