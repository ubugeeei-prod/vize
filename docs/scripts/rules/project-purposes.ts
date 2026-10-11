/** Documentation qualifications for inherited diagnostic text; runtime messages stay unchanged. */
const qualifiedPurposes: Readonly<Record<string, readonly [en: string, ja: string]>> = {
  "array-mutation": [
    "This producer-free contract uses a historical Vue 2.7 array-index assignment scenario. Vue 2 does not notify updates for direct index assignment; Vue 3 tracks such assignments. The Bad/Good files illustrate the Vue 2 repair, not a Vue 3 defect.",
    "現在の生成元がないこの契約では、旧 Vue 2.7 の配列のインデックス代入を例にしています。Vue 2 は直接のインデックス代入による更新を通知しませんが、Vue 3 はその代入を追跡します。悪い例・良い例は Vue 2 の修正を示すもので、Vue 3 の不具合を示す例ではありません。",
  ],
  "pinia-getter": [
    "This contract illustrates copying a Pinia getter's current primitive value during setup, leaving a snapshot that no longer follows updates. Direct getter reads within a tracked computation or template remain reactive; storeToRefs() is the extraction repair shown here.",
    "この契約の例では、setup で Pinia の getter から現在のプリミティブ値をコピーし、更新に追従しないスナップショットを作っています。追跡される計算やテンプレートで getter を直接読む場合はリアクティビティを保てます。この例では `storeToRefs()` を使って値を取り出します。",
  ],
  "setup-context-violation": [
    "This experimental analyzer scenario places ref(0) at normal script module scope rather than per-instance setup. Vue permits this reactive state; the example requires each component instance to own its own count. The CLI does not emit this individual code.",
    "実験的な解析器の例では、インスタンスごとの setup ではなく通常の script のモジュールスコープに `ref(0)` を置いています。Vue ではこのようなリアクティブな状態を作れますが、この例ではインスタンスごとに count を持つことを前提とします。CLI はこの個別の診断コードを生成しません。",
  ],
  "object-identity-comparison": [
    "A raw object and its reactive proxy have distinct object identities, and toRaw() returns the original object. This contract illustrates comparing stable record IDs when record identity is intended.",
    "元のオブジェクトとそのリアクティブな Proxy は別の同一性を持ち、`toRaw()` は元のオブジェクトを返します。この契約の例は、レコードとして同じものかを判定するために安定した ID を比較する方法を示します。",
  ],
  "watcheffect-async": [
    "The async watchEffect shown here has no invalidation cleanup or stale-response guard. Vue supports cleanup for watchEffect; the Good example uses an explicit watch source and cancels invalidated work.",
    "この例の非同期の `watchEffect` には、無効化時のクリーンアップや古い応答を防ぐ処理がありません。Vue の `watchEffect` はクリーンアップを扱えます。良い例では `watch` で監視対象を明示し、無効化された処理を中止します。",
  ],
};

export function qualifiedProjectPurpose(name: string, inherited: string, ja: boolean) {
  return qualifiedPurposes[name]?.[ja ? 1 : 0] ?? inherited;
}
