import assert from "node:assert/strict";
import { CORSA_BIN, VIZE_BIN, requireVizeAndCorsaBins } from "../../../_helpers/apps.ts";

if (process.argv[2] === "require") requireVizeAndCorsaBins();
else {
  assert.equal(VIZE_BIN, process.env.VIZE_BIN);
  assert.equal(CORSA_BIN, process.env.CORSA_BIN);
}
