import assert from "node:assert/strict";
import { textFile } from "../shared/files.ts";
const text = textFile("evidence/send-law.log");
assert.ok(text.includes("running 1 test") && text.includes("1 passed; 0 failed"));
