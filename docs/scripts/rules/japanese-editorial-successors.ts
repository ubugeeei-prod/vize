import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export interface EditorialCase {
  case: string;
  rule: string;
  section: "purpose" | "bad" | "good";
  before: string;
  after: string;
}

/** Only these five reviewed paragraphs move from the purpose to a named section. */
export const japaneseEditorialPlacementSuccessors = {
  B01: {
    rule: "vize:croquis/cf/array-mutation",
    purpose: "Vue 2.7 の配列のインデックス更新に、通知される操作を使います。",
    paragraphSha256: "25cfd302677cc9e0ebf1979407e756f591358880fbe40342bf056423ccb19421",
  },
  B02: {
    rule: "vize:croquis/cf/pinia-getter",
    purpose: "setup で Pinia の getter を更新に追従しない値へコピーする操作を避けます。",
    paragraphSha256: "1390284e482ce6e6a8659b20529850e2fea98d2fbd65f2180821d1675f361c58",
  },
  B03: {
    rule: "vize:croquis/cf/setup-context-violation",
    purpose: "インスタンスごとに持つリアクティブな状態を setup 内で作ります。",
    paragraphSha256: "1d62e657af7bfb48aa3f3d39d8266aee48ad3f3be90fdd638253d2c807b591d3",
  },
  B04: {
    rule: "vize:croquis/cf/object-identity-comparison",
    purpose: "オブジェクトと Proxy が表すレコードの同一性を、安定した ID で比較します。",
    paragraphSha256: "7d5ec7e79584e073b7def7310c8f8bfc0acae2d1c95afa360ae3a0a7a2f1c367",
  },
  B05: {
    rule: "vize:croquis/cf/watcheffect-async",
    purpose: "非同期の watchEffect で無効になった処理を中止し、古い応答を反映しないようにします。",
    paragraphSha256: "769dca8d96b04f0e230750df5bdc3f5b13f82e6a278fbad216069811f9894911",
  },
} as const;

export function japaneseEditorialReferenceCases(entries: readonly EditorialCase[]) {
  for (const id of Object.keys(japaneseEditorialPlacementSuccessors))
    assert.equal(entries.filter((entry) => entry.case === id).length, 1, `${id}: one frozen case`);
  return entries.map((entry) => {
    const successor =
      japaneseEditorialPlacementSuccessors[
        entry.case as keyof typeof japaneseEditorialPlacementSuccessors
      ];
    if (!successor) return { ...entry, targetSection: entry.section };
    assert.equal(entry.rule, successor.rule, `${entry.case}: original rule identity`);
    assert.equal(entry.section, "purpose", `${entry.case}: original paragraph placement`);
    assert.equal(
      createHash("sha256").update(entry.after).digest("hex"),
      successor.paragraphSha256,
      `${entry.case}: complete original reviewed paragraph`,
    );
    return {
      ...entry,
      targetSection: "qualification" as const,
      qualificationHeading: "範囲と制約",
      expectedPurpose: successor.purpose,
    };
  });
}
