import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { ownedCorsaSource } from "./owned-corsa-source-custody.mjs";

// The literal 237 before cells remain unchanged at this independently reviewed main.
// This changes only source context; the full actual pair still gates projection.
export const ownedCorsaCurrentBaseline = Object.freeze({
  revision: "ace25e4f9f2959d97941653decc55619bf154316",
  tree: "d4d9454052ce58961a9b4d066596cded1f7d0742",
  rawCommitSHA256: "344769a70de5894699b30d2329268cf8eff0529a324fe20aa121e83466f8839e",
  rawCommitBytes: 1095,
});

// This actual merged main retains every complete literal 237 before cell.
export const ownedCorsaMergedMainBaseline = Object.freeze({
  revision: "a42878a9af4a9e79e42cc56bbcd55fc0c66400e3",
  tree: "eeab9af4d02ead0c0de04db3595ea11b5eb914b8",
  rawCommitSHA256: "b9823c6e8e90463429e75bb117be58267aa6673eb368089be60e152d0c95c0cb",
  rawCommitBytes: 1388,
});

export const ownedCorsaLatestMergedBaseline = Object.freeze({
  revision: "d5a25a89540198f9cfb50cb9e2e6e74a24d6d14e",
  tree: "a0b9b88acce3ae75236d017b26f124e7859c77b9",
  rawCommitSHA256: "adafd1908c53a13e284614cef739561fd16c6081275827782206fde9569e3cf5",
  rawCommitBytes: 3649,
});

export const ownedCorsaDocumentationMainBaseline = Object.freeze({
  revision: "02731ffc046b08e947e087b80da0b745884e1229",
  tree: "37ac869ddabe51769ee9a2ede35765596fdf76b7",
  rawCommitSHA256: "52c81110eae81453efac5e879659270699cd022fc0e69679b3231f6ae2240cf1",
  rawCommitBytes: 1088,
});

export const ownedCorsaRecoveryMainBaseline = Object.freeze({
  revision: "f20740ef6b0958ddb0bbbd64fd1b38210ea5e207",
  tree: "d42e1a13f69c4bfc48ee9b4a8bd0c46671abd18c",
  rawCommitSHA256: "9579d0c3a8467c1666a9e5360c3af9a28ce63068922f6da606dd74658ca4dbe0",
  rawCommitBytes: 1101,
});

export const ownedCorsaMuseaDocumentationMainBaseline = Object.freeze({
  revision: "a93e78147b0990c813e965a7868b32dda341eb77",
  tree: "5ff35375c2c2e24654139227e53ee0cbb357116c",
  rawCommitSHA256: "b7d4981e01688fd26fe3124a2291d96fde2a8e4459762cb493d0299ea11de1aa",
  rawCommitBytes: 1083,
});

// Actual accessibility-documentation main retains all complete 237 before cells.
export const ownedCorsaAccessibilityDocumentationMainBaseline = Object.freeze({
  revision: "db258f5e1c75199aa3ee87e780612652156ad41c",
  tree: "0b7cf2d08dde18605aa0c27fd89babc4c41d70e2",
  rawCommitSHA256: "c8a07746842c8b96465c68abc540b1d20a001e10a3df66f70469052c49577887",
  rawCommitBytes: 1093,
});

// The actual merged NoMutating main retains every complete original237 before cell.
export const ownedCorsaNoMutatingPropsMainBaseline = Object.freeze({
  revision: "7d6e0350202643010db3a0f474eb235c8f1042c3",
  tree: "03ffeac65c16dd3d310adfde7586d15e6db7d96c",
  rawCommitSHA256: "bff0a78beb027ff225a72135117c5c5117c3061d5585e798a596009d1d175279",
  rawCommitBytes: 1940,
});

// Actual component documentation main leaves all complete237 before cells literal.
export const ownedCorsaJapaneseComponentDocumentationMainBaseline = Object.freeze({
  revision: "3601d6810158d77d63ad995879bf9ec746cd7547",
  tree: "982528ca5b4c46d7191a3a57fd465114c564dc88",
  rawCommitSHA256: "870c09ee56daaca7c4005140dc5ec63337e6be94beee1d0a5b26388bfdc4766d",
  rawCommitBytes: 1190,
});

// Current merged main is a separate source identity; all237 actual pairs remain mandatory.
export const ownedCorsaNativeGatesMainBaseline = Object.freeze({
  revision: "bd5702c0a4d5fb3bb64487794fc45f6f876e6e38",
  tree: "4dfec533534c3c15e5be3678423d297a44ab229e",
  rawCommitSHA256: "a45bd8226630844bf461db4ee8f7bb651acffaa087cdb0a0288d55c9959da163",
  rawCommitBytes: 1265,
});

export function retainedOwnedCorsaBaseline(root, baseline) {
  // Unknown contexts stay literal and the original complete qualifier refuses them.
  const context =
    baseline === ownedCorsaCurrentBaseline.revision
      ? ownedCorsaCurrentBaseline
      : baseline === ownedCorsaMergedMainBaseline.revision
        ? ownedCorsaMergedMainBaseline
        : baseline === ownedCorsaLatestMergedBaseline.revision
          ? ownedCorsaLatestMergedBaseline
          : baseline === ownedCorsaDocumentationMainBaseline.revision
            ? ownedCorsaDocumentationMainBaseline
            : baseline === ownedCorsaRecoveryMainBaseline.revision
              ? ownedCorsaRecoveryMainBaseline
              : baseline === ownedCorsaMuseaDocumentationMainBaseline.revision
                ? ownedCorsaMuseaDocumentationMainBaseline
                : baseline === ownedCorsaAccessibilityDocumentationMainBaseline.revision
                  ? ownedCorsaAccessibilityDocumentationMainBaseline
                  : baseline === ownedCorsaNoMutatingPropsMainBaseline.revision
                    ? ownedCorsaNoMutatingPropsMainBaseline
                    : baseline === ownedCorsaJapaneseComponentDocumentationMainBaseline.revision
                      ? ownedCorsaJapaneseComponentDocumentationMainBaseline
                      : baseline === ownedCorsaNativeGatesMainBaseline.revision
                        ? ownedCorsaNativeGatesMainBaseline
                        : null;
  if (!context) return baseline;
  const git = (args) =>
    execFileSync("git", ["--no-replace-objects", ...args], {
      cwd: root,
      env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
      maxBuffer: 1024 * 1024,
    });
  assert.equal(git(["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  const raw = git(["cat-file", "commit", baseline]);
  assert.equal(raw.length, context.rawCommitBytes);
  assert.equal(createHash("sha256").update(raw).digest("hex"), context.rawCommitSHA256);
  assert.equal(raw.toString().split("\n")[0], "tree " + context.tree);
  return ownedCorsaSource.baseline;
}
