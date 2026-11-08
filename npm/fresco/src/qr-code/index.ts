/** Optional, provider-driven QR code entry point. */

import { computed, defineComponent, h, type PropType } from "vue";

export type QrCodeErrorCorrection = "low" | "medium" | "quartile" | "high";
export type QrCodeMatrix = readonly (readonly boolean[])[];

export interface QrCodeProviderOptions {
  errorCorrection: QrCodeErrorCorrection;
}

export type QrCodeProvider = (
  value: string,
  options: Readonly<QrCodeProviderOptions>,
) => QrCodeMatrix;

export interface QrCodeProps {
  value: string;
  provider: QrCodeProvider;
  errorCorrection?: QrCodeErrorCorrection;
  quietZone?: number;
  cellWidth?: number;
  dark?: string;
  light?: string;
  fg?: string;
  bg?: string;
  invert?: boolean;
}

/** Preserve a provider's exact public type without wrapping its implementation. */
export function defineQrCodeProvider(provider: QrCodeProvider): QrCodeProvider {
  return provider;
}

function normalizedInteger(value: number, fallback: number): number {
  if (!Number.isFinite(value)) return fallback;
  return Math.max(0, Math.trunc(value));
}

function validateMatrix(matrix: QrCodeMatrix): boolean[][] {
  if (matrix.length === 0) {
    throw new Error("FRESCO_QR_MATRIX_EMPTY: provider returned an empty matrix");
  }
  const size = matrix.length;
  return matrix.map((row, rowIndex) => {
    if (row.length !== size) {
      throw new Error(
        `FRESCO_QR_MATRIX_SQUARE: row ${rowIndex} has ${row.length} cells; expected ${size}`,
      );
    }
    return row.map((cell, columnIndex) => {
      if (typeof cell !== "boolean") {
        throw new Error(
          `FRESCO_QR_MATRIX_CELL: row ${rowIndex}, column ${columnIndex} is not boolean`,
        );
      }
      return cell;
    });
  });
}

/** Render a provider matrix without bundling a QR encoder in Fresco. */
export const QrCode = defineComponent({
  name: "QrCode",
  props: {
    value: { type: String, required: true },
    provider: { type: Function as PropType<QrCodeProvider>, required: true },
    errorCorrection: {
      type: String as PropType<QrCodeErrorCorrection>,
      default: "medium",
    },
    quietZone: { type: Number, default: 2 },
    cellWidth: { type: Number, default: 2 },
    dark: { type: String, default: "█" },
    light: { type: String, default: " " },
    fg: String,
    bg: String,
    invert: Boolean,
  },
  setup(props) {
    const matrix = computed(() =>
      validateMatrix(
        props.provider(props.value, {
          errorCorrection: props.errorCorrection,
        }),
      ),
    );
    const rows = computed(() => {
      const quietZone = normalizedInteger(props.quietZone, 2);
      const cellWidth = Math.max(1, normalizedInteger(props.cellWidth, 2));
      const dark = (props.invert ? props.light : props.dark).repeat(cellWidth);
      const light = (props.invert ? props.dark : props.light).repeat(cellWidth);
      const quietRow = light.repeat(matrix.value.length + quietZone * 2);
      const horizontalPadding = light.repeat(quietZone);
      return [
        ...Array<string>(quietZone).fill(quietRow),
        ...matrix.value.map(
          (row) =>
            `${horizontalPadding}${row.map((cell) => (cell ? dark : light)).join("")}${horizontalPadding}`,
        ),
        ...Array<string>(quietZone).fill(quietRow),
      ];
    });

    return () =>
      h(
        "box",
        {
          style: { flexDirection: "column" },
          "aria-label": `QR code for ${props.value}`,
        },
        rows.value.map((row, index) =>
          h("text", { key: index, text: row, fg: props.fg, bg: props.bg }),
        ),
      );
  },
});
