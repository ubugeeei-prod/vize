import assert from "node:assert/strict";
import { assertPreservedCallbackPacket } from "../../../tools/support/compat/content-mapper/callback-owner-audit.ts";

export function assertJoinedVerbatimRunCustody(): void {
  const source = "alpha\nbeta";
  const before = {
    text: source,
    mappings: [
      [0, 5, 0, 5, 0, 7],
      [6, 4, 6, 4, 0, 7],
    ],
    diagnosticDirectives: {
      directives: [[0, 0, 5, 6, 0, 0]],
      unusedExpectDirectiveDiagnostics: [],
    },
  };
  assertPreservedCallbackPacket(
    before,
    {
      ...before,
      mappings: [[0, 10, 0, 10, 0, 7]],
      diagnosticDirectives: { ...before.diagnosticDirectives, directives: [] },
    },
    source,
  );
  assert.throws(
    () =>
      assertPreservedCallbackPacket(
        before,
        { ...before, mappings: [[0, 10, 0, 10, 0, 5]] },
        source,
      ),
    assert.AssertionError,
  );

  const crlfSource = "alpha\r\nbeta";
  const crlf = {
    text: crlfSource,
    mappings: [
      [0, 5, 0, 5, 0, 7],
      [7, 4, 7, 4, 0, 7],
    ],
    diagnosticDirectives: {
      directives: [[0, 0, 5, 7, 0, 0]],
      unusedExpectDirectiveDiagnostics: [],
    },
  };
  assertPreservedCallbackPacket(crlf, structuredClone(crlf), crlfSource);
  assert.throws(
    () =>
      assertPreservedCallbackPacket(
        crlf,
        { ...crlf, mappings: [[0, crlfSource.length, 0, crlfSource.length, 0, 7]] },
        crlfSource,
      ),
    assert.AssertionError,
  );
}
