import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const metadata = JSON.parse(readFileSync("provider-metadata.json", "utf8")) as {
  gitHead: string;
  "dist.integrity": string;
};
assert.equal(metadata.gitHead, "3e9236361fa97a9e6c685f4833ac457ae2d8c50a");
assert.match(metadata["dist.integrity"], /^sha512-/u);
