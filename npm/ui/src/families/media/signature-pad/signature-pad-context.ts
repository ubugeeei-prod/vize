import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  SignaturePadPressureMode,
  SignaturePadSlotState,
  SignaturePoint,
  SignatureStroke,
  SignatureStrokeOptions,
  SignatureValue,
} from "./signature-pad-types.ts";

/** Shared state and actions for the SignaturePad compound parts. */
export interface SignaturePadContextValue {
  readonly id: ComputedRef<string>;
  readonly value: ComputedRef<SignatureValue>;
  readonly slotState: ComputedRef<SignaturePadSlotState>;
  readonly width: ComputedRef<number>;
  readonly height: ComputedRef<number>;
  readonly strokeOptions: ComputedRef<SignatureStrokeOptions>;
  readonly pressureMode: ComputedRef<SignaturePadPressureMode>;
  readonly interactive: ComputedRef<boolean>;
  readonly liveStroke: Readonly<ShallowRef<SignatureStroke | null>>;
  readonly startStroke: (point: SignaturePoint) => boolean;
  readonly extendStroke: (points: readonly SignaturePoint[]) => void;
  readonly finishStroke: () => void;
  readonly cancelStroke: () => void;
  readonly clear: () => boolean;
  readonly undo: () => boolean;
  readonly redo: () => boolean;
}

export const signaturePadContext = createContext<SignaturePadContextValue>("SignaturePad");
