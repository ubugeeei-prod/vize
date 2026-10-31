#!/usr/bin/env node
/** Node-only observer fixture, never a native compiler acceptance provider. */
import fs from "node:fs";
const buildInfoAt = process.argv.indexOf("--tsBuildInfoFile");
if (buildInfoAt >= 0) fs.writeFileSync(process.argv[buildInfoAt + 1], "owned-output\n");
if (process.env.VIZE_TEST_MUTATE) fs.appendFileSync(process.env.VIZE_TEST_MUTATE, "changed\n");
process.stdout.write("src/Probe.vue.ts(1,1): error TS5097: exact fixture diagnostic\n");
process.stderr.write("exact fixture stderr\n");
process.exitCode = 2;
