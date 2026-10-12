import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const priorNuxtSource = "4e4c8c977d7052edbb6a2b42808ddc6e7b59f3bf";
export const priorNuxtConstructor = "npm/framework/nuxt/src/index.ts";
const sha = (body) => createHash("sha256").update(body).digest("hex");
const before =
  "      vize({ ...compilerOptions, nuxtPageMeta: true, ssrModuleIdRoot: nuxt.options.srcDir }),";
const after =
  '      vize({ ...compilerOptions, nuxtPageMeta: true, ssrModuleIdRoot: nuxt.options.srcDir }, {}, "nuxt"),';

/** The historical Nuxt algorithm keeps every byte outside its known adapter call. */
export function adaptPriorNuxtHostContext(body, source) {
  assert.equal(source, priorNuxtSource);
  assert.ok(Buffer.isBuffer(body));
  assert.equal(body.length, 24345);
  assert.equal(sha(body), "858c488fef903831fa4dcf19c1a534f71dbfc0a75b5e6ef2b7474a49913eee44");
  const text = body.toString("utf8");
  assert.deepEqual(Buffer.from(text), body);
  assert.equal(text.split(before).length, 2);
  const adapted = Buffer.from(text.replace(before, after));
  assert.equal(adapted.toString().split(after).length, 2);
  assert.deepEqual(Buffer.from(adapted.toString().replace(after, before)), body);
  return adapted;
}

/** Preserve both complete source bodies before the original historical package build. */
export function preparePriorNuxtHostContext(root, source, artifacts) {
  const target = path.join(root, priorNuxtConstructor);
  const status = fs.lstatSync(target);
  assert.ok(status.isFile() && !status.isSymbolicLink() && !(status.mode & 0o111));
  assert.equal(fs.realpathSync(target), path.join(fs.realpathSync(root), priorNuxtConstructor));
  const original = fs.readFileSync(target);
  const adapted = adaptPriorNuxtHostContext(original, source);
  for (const { name, body } of [
    { name: "before", body: original },
    { name: "after", body: adapted },
  ])
    fs.writeFileSync(path.join(artifacts, `prior-nuxt-host-context.${name}.ts`), body, {
      flag: "wx",
    });
  fs.writeFileSync(
    path.join(artifacts, "prior-nuxt-host-context.json"),
    JSON.stringify(
      {
        source,
        path: priorNuxtConstructor,
        before: { bytes: original.length, sha256: sha(original) },
        after: { bytes: adapted.length, sha256: sha(adapted) },
        change: "known historical Nuxt constructor supplies the private host configuration context",
        originalAlgorithmWholeInverse: true,
        credit:
          "historical adapter compatibility only; no current production or consumer acceptance",
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  assert.deepEqual(fs.readFileSync(target), original);
  fs.writeFileSync(target, adapted);
  assert.deepEqual(fs.readFileSync(target), adapted);
}
