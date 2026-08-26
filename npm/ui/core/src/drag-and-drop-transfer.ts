import type { DragPayload } from "./drag-and-drop-types.ts";

const invalidTransferDiagnostic = "VIZE_UI_DRAG_AND_DROP_TRANSFER";

/** Structured data-transfer format written next to the plain-text projection. */
export const DRAG_TRANSFER_TYPE = "application/vnd.vize-ui.drag+json";

function validatePayload(payload: DragPayload): void {
  if (
    !payload ||
    typeof payload.kind !== "string" ||
    payload.kind.length === 0 ||
    !("data" in payload)
  ) {
    throw new TypeError(`${invalidTransferDiagnostic}: payload must carry a kind and data`);
  }
  if (payload.plainText !== undefined && typeof payload.plainText !== "string") {
    throw new TypeError(`${invalidTransferDiagnostic}: plainText must be a string`);
  }
}

/**
 * Serialize one typed payload onto a `DataTransfer`.
 *
 * The structured payload is written under {@link DRAG_TRANSFER_TYPE}; the
 * optional `plainText` projection is mirrored to `text/plain` so external
 * applications receive a meaningful representation.
 */
export function writeDragTransfer(dataTransfer: DataTransfer, payload: DragPayload): void {
  validatePayload(payload);
  dataTransfer.setData(
    DRAG_TRANSFER_TYPE,
    JSON.stringify({ kind: payload.kind, data: payload.data }),
  );
  if (payload.plainText !== undefined) dataTransfer.setData("text/plain", payload.plainText);
}

/**
 * Deserialize a typed payload from a `DataTransfer`.
 *
 * @returns The structured payload, or `null` when the transfer does not carry
 * a well-formed {@link DRAG_TRANSFER_TYPE} entry.
 */
export function readDragTransfer<Data = unknown>(
  dataTransfer: DataTransfer,
): DragPayload<Data> | null {
  const serialized = dataTransfer.getData(DRAG_TRANSFER_TYPE);
  if (!serialized) return null;
  let parsed: unknown;
  try {
    parsed = JSON.parse(serialized);
  } catch {
    return null;
  }
  if (!parsed || typeof parsed !== "object") return null;
  const { kind, data } = parsed as { kind?: unknown; data?: unknown };
  if (typeof kind !== "string" || kind.length === 0 || !("data" in parsed)) return null;
  const plainText = dataTransfer.getData("text/plain");
  return Object.freeze(
    plainText ? { kind, data: data as Data, plainText } : { kind, data: data as Data },
  );
}

/**
 * Serialize one typed payload onto a clipboard event during `copy` or `cut`.
 *
 * The event's default action is prevented so the written data wins.
 *
 * @returns `true` when the event exposed writable clipboard data.
 */
export function writeClipboardTransfer(event: ClipboardEvent, payload: DragPayload): boolean {
  validatePayload(payload);
  const dataTransfer = event.clipboardData;
  if (!dataTransfer) return false;
  event.preventDefault();
  writeDragTransfer(dataTransfer, payload);
  return true;
}

/**
 * Deserialize a typed payload from a clipboard event during `paste`.
 *
 * @returns The structured payload, or `null` when the clipboard does not carry
 * a well-formed {@link DRAG_TRANSFER_TYPE} entry.
 */
export function readClipboardTransfer<Data = unknown>(
  event: ClipboardEvent,
): DragPayload<Data> | null {
  const dataTransfer = event.clipboardData;
  return dataTransfer ? readDragTransfer<Data>(dataTransfer) : null;
}
