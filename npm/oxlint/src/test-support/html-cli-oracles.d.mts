// Types for the unchanged original whole-packet test oracle. No runtime/package binding is assigned here.
import type {
  HtmlFixture,
  HtmlCapture,
  ExpectedPacket,
  HtmlRow,
} from "../../../../tests/tooling/support/oxlint-installed-html-types.ts";
export function readEvents(file: string): unknown[];
export function plainHostEnvironment(original: NodeJS.ProcessEnv): NodeJS.ProcessEnv;
export function snapshot(root: string): unknown[][];
export const fixtures: HtmlFixture[];
export function completeCase(
  capture: HtmlCapture,
  nativeCalls: readonly unknown[],
  hostPhases: readonly unknown[],
): void;
export function finishCapture(capture: HtmlCapture): void;
export function expectedOriginalRows(
  fixture: HtmlFixture,
  packet: ExpectedPacket,
  root: string,
  stockRows: HtmlRow[],
): HtmlRow[];
export function assertStandaloneOutput(
  fixture: HtmlFixture,
  format: string,
  output: string,
  repository: string,
): void;
export function expectedRows(
  file: string,
  packet: ExpectedPacket,
  source: Buffer,
  helpLevel?: string,
): HtmlRow[];
