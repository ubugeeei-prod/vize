/** Maximum view nodes accepted by the experimental first host. */
export const NATIVE_HOST_MAXIMUM_NODES = 1024;
/** Maximum rooted view depth accepted by the experimental first host. */
export const NATIVE_HOST_MAXIMUM_DEPTH = 64;
/** Maximum UTF-16 length of one native label or text value. */
export const NATIVE_HOST_MAXIMUM_TEXT = 4096;
/** Maximum native layout spacing in points. */
export const NATIVE_HOST_MAXIMUM_SPACING = 4096;

/** Native vertical-stack layout options supported by the first host. */
export interface NativeViewProps {
  /**
   * Space between native child views, from 0 through 4096 points.
   *
   * @default 0
   */
  readonly gap?: number;
  /**
   * Equal native content insets, from 0 through 4096 points.
   *
   * @default 0
   */
  readonly padding?: number;
}

/** Text semantics shared by the SFC primitive and the native label adapter. */
export interface NativeTextProps {
  /** Text and accessible name, at most 4096 UTF-16 code units. */
  readonly text: string;
}

/** Activation semantics shared by the SFC primitive and native button adapter. */
export interface NativeButtonProps {
  /** Non-empty visible and accessible label, at most 4096 UTF-16 code units. */
  readonly label: string;
}

/** One target-independent node delivered to the first native host. */
export type NativeViewNode =
  | {
      readonly id: number;
      readonly kind: "view";
      readonly children: readonly number[];
      readonly gap: number;
      readonly padding: number;
    }
  | {
      readonly id: number;
      readonly kind: "text";
      readonly children: readonly number[];
      readonly text: string;
    }
  | {
      readonly id: number;
      readonly kind: "button";
      readonly children: readonly number[];
      readonly label: string;
    };

/** An authoritative view-tree update, validated before native installation. */
export interface NativeViewTree {
  readonly format: "vize.native-view-tree";
  readonly formatVersion: 1;
  readonly sequence: number;
  readonly root: number;
  readonly nodes: readonly NativeViewNode[];
}

/** A deterministic refusal delivered to the native development surface. */
export interface NativeHostDiagnostic {
  readonly format: "vize.native-diagnostic";
  readonly formatVersion: 1;
  readonly code: string;
  readonly message: string;
}

/** Transport implemented by a real native host or an isolated contract test. */
export interface NativeHostBridge {
  /** Deliver a validated tree or a refusal without exposing native engine objects. */
  readonly postMessage: (message: NativeViewTree | NativeHostDiagnostic) => void;
}

/** A host-side failure whose stable code may cross the native bridge. */
export class NativeHostFailure extends Error {
  /** Stable machine-readable native boundary refusal. */
  readonly code: string;
  /** Creates a deterministic native boundary refusal. */
  constructor(code: string, message: string) {
    super(message);
    this.code = code;
    this.name = "NativeHostFailure";
  }
}
