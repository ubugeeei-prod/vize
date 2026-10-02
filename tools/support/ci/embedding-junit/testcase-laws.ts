import assert from "node:assert/strict";
import { actualTestcase } from "./testcase.ts";

const name = "remarks_preserve_zero_cost_and_the_positive_control", binary = "vize_l0::remark_zero_cost";
const pass = `<testcase name="${name}" classname="${binary}" time="0.002"/>`;
assert.equal(actualTestcase(`<testsuite>${pass}</testsuite>`, name, binary).testcase, pass);
const decoy = `<testcase name="foreign" classname="foreign"><system-out>name="${name}" classname="${binary}"</system-out></testcase>`;
assert.throws(() => actualTestcase(decoy, name, binary), /one exact opening-tag/);
assert.throws(() => actualTestcase(pass + pass, name, binary), /one exact opening-tag/);
assert.throws(() => actualTestcase(`<testcase name="${name}" classname="${binary}"><skipped/></testcase>`, name, binary), /genuinely passed/);
assert.throws(() => actualTestcase(`<testcase name="${name}" classname="${binary}"><failure/></testcase>`, name, binary), /genuinely passed/);
assert.throws(() => actualTestcase(`<testcase wrong-name="${name}" classname="${binary}"/>`, name, binary), /one exact opening-tag/);
assert.throws(() => actualTestcase(`<testcase name="foreign" name="${name}" classname="${binary}"/>`, name, binary), /unique actual attributes/);
console.log("actual opening-tag identity PASS; six meaningful counterfeit/skip/failure controls refused");
