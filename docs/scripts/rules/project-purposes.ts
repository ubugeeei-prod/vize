/** Documentation purposes qualify inherited runtime wording without changing runtime messages. */
const concisePurposes: Readonly<Record<string, readonly [en: string, ja: string]>> = {
  "array-mutation": [
    "Use observable mutations for array index updates in Vue 2.7.",
    "Vue 2.7 の配列のインデックス更新に、通知される操作を使います。",
  ],
  "pinia-getter": [
    "Avoid setup-time snapshots of Pinia getter values.",
    "setup で Pinia の getter を更新に追従しない値へコピーする操作を避けます。",
  ],
  "setup-context-violation": [
    "Keep component-owned reactive state inside per-instance setup.",
    "インスタンスごとに持つリアクティブな状態を setup 内で作ります。",
  ],
  "object-identity-comparison": [
    "Compare stable record IDs across raw objects and reactive proxies.",
    "オブジェクトと Proxy が表すレコードの同一性を、安定した ID で比較します。",
  ],
  "watcheffect-async": [
    "Guard async watchEffect work against invalidation and stale responses.",
    "非同期の watchEffect で無効になった処理を中止し、古い応答を反映しないようにします。",
  ],
};

/** Complete reviewed qualifications; retain the original paragraphs verbatim. */
const scopeQualifications: Readonly<Record<string, readonly [en: string, ja: string]>> = {
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

const documentationNotes: Readonly<Record<string, readonly [en: string, ja: string]>> = {
  "hydration-risk": [
    "This code groups several reactivity findings, including a prop copied into a ref. It does not imply that every Date.now() expression is detected by the cross-file pass.",
    "この診断コードは、prop を ref にコピーする操作など、リアクティビティに関する複数の検出結果をまとめています。ファイル間検査がすべての `Date.now()` 式を検出するという意味ではありません。",
  ],
};

export function projectPurpose(name: string, inherited: string, ja: boolean) {
  return concisePurposes[name]?.[ja ? 1 : 0] ?? inherited;
}

export function projectScopeQualification(name: string, ja: boolean) {
  return scopeQualifications[name]?.[ja ? 1 : 0];
}

export function projectDocumentationNote(name: string, ja: boolean) {
  return documentationNotes[name]?.[ja ? 1 : 0];
}
