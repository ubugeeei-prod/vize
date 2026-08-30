import type { CSSProperties } from "vue";

import type { PrimitiveAs } from "../../../primitive.ts";

/** Directional layout hint used by Toolbar arrow-key roving focus. */
export type ToolbarOrientation = "horizontal" | "vertical";

/** Reading direction used to map horizontal Toolbar arrow keys. */
export type ToolbarDirection = "ltr" | "rtl";

/** Stable root state exposed for consumer-owned styling and tests. */
export type ToolbarState = "disabled" | "idle";

/** Stable item state exposed for consumer-owned styling and tests. */
export type ToolbarItemState = "disabled" | "idle";

/** Stable part names emitted by the Toolbar family. */
export type ToolbarPart = "item" | "root";

/** Stable `data-vize-ui` values emitted by the Toolbar family. */
export type ToolbarDataName = "toolbar" | "toolbar-item";

/** Stable data attributes emitted by one or more Toolbar components. */
export type ToolbarDataAttribute =
  | "data-disabled"
  | "data-orientation"
  | "data-roving-focus"
  | "data-state"
  | "data-value"
  | "data-vize-ui";

/** CSS custom properties authored inline by the Toolbar root. */
export type ToolbarCssCustomProperty = "--vize-ui-toolbar-orientation";

/** Inline style contract applied to the Toolbar root. */
export interface ToolbarStyle extends Readonly<CSSProperties> {
  /** Consumer-overridable orientation token for layout recipes. */
  readonly "--vize-ui-toolbar-orientation": ToolbarOrientation;
}

/** Public props accepted by the root Toolbar component. */
export interface ToolbarProps {
  /**
   * Native element, custom element, or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Disable every item and remove the toolbar from roving focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Directional layout hint used by arrow-key roving focus.
   *
   * @default "horizontal"
   */
  readonly orientation?: ToolbarOrientation;

  /**
   * Reading direction used to map horizontal arrow-key navigation.
   *
   * @default "ltr"
   */
  readonly dir?: ToolbarDirection;

  /**
   * Whether arrow-key navigation wraps at the first and last enabled item.
   *
   * @default true
   */
  readonly loop?: boolean;

  /**
   * Whether items participate in a single-tabstop roving focus model.
   *
   * @default true
   */
  readonly rovingFocus?: boolean;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the toolbar.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the toolbar.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}

/** Props accepted by ToolbarItem. */
export interface ToolbarItemProps {
  /**
   * Native element, custom element, or component to render.
   *
   * @default "button"
   */
  readonly as?: PrimitiveAs;

  /**
   * Whether the rendered target already implements native button semantics.
   *
   * @default auto
   */
  readonly native?: boolean;

  /**
   * Native button submission behavior.
   *
   * @default "button"
   */
  readonly type?: "button" | "reset" | "submit";

  /**
   * Stable item value emitted by item and toolbar press events.
   *
   * @default required
   */
  readonly value: string;

  /**
   * Disable this item while preserving the rest of the toolbar.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label this item.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe this item.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}

/** Events emitted by the root Toolbar component. */
export interface ToolbarEmits {
  /** Fired after an enabled item is activated by pointer or keyboard. */
  press: [value: string, nativeEvent: MouseEvent];
}

/** Events emitted by ToolbarItem. */
export interface ToolbarItemEmits {
  /** Fired after user activation reaches this enabled item. */
  press: [value: string, nativeEvent: MouseEvent];
}

/** State exposed to the Toolbar default slot. */
export interface ToolbarSlotState {
  /** Whether the toolbar suppresses every item activation. */
  readonly disabled: boolean;

  /** Directional layout hint used by arrow-key roving focus. */
  readonly orientation: ToolbarOrientation;

  /** Reading direction used by horizontal arrow-key roving focus. */
  readonly dir: ToolbarDirection;

  /** Whether items participate in a single-tabstop roving focus model. */
  readonly rovingFocus: boolean;

  /** Stable state token for styling and tests. */
  readonly state: ToolbarState;

  /** Inline native style object applied to the host. */
  readonly style: ToolbarStyle;
}

/** State exposed to a ToolbarItem slot. */
export interface ToolbarItemSlotState {
  /** Item value used by the toolbar navigation and press contract. */
  readonly value: string;

  /** Whether this item or its toolbar suppresses activation. */
  readonly disabled: boolean;

  /** Directional layout hint inherited from the toolbar. */
  readonly orientation: ToolbarOrientation;

  /** Reading direction inherited from the toolbar. */
  readonly dir: ToolbarDirection;

  /** Stable state token for styling and tests. */
  readonly state: ToolbarItemState;
}
