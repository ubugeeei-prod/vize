/** Caveats for an example. They do not replace the rule's purpose sentence. */
const documentationNotes: Readonly<Record<string, readonly [en: string, ja: string]>> = {
  "hydration-risk": [
    "This code groups several reactivity findings, including a prop copied into a ref. It does not imply that every Date.now() expression is detected by the cross-file pass.",
    "この診断コードは、prop を ref にコピーする操作など、リアクティビティに関する複数の検出結果をまとめています。ファイル間検査がすべての `Date.now()` 式を検出するという意味ではありません。",
  ],
  "setup-context-violation": [
    "Vue permits this reactive state; the example requires each component instance to own its own count.",
    "Vue ではこのようなリアクティブな状態を作れますが、この例ではインスタンスごとに count を持つことを前提とします。",
  ],
  "watcheffect-async": [
    "Vue supports cleanup for watchEffect; the Good example uses an explicit watch source and cancels invalidated work.",
    "Vue の `watchEffect` はクリーンアップを扱えます。良い例では `watch` で監視対象を明示し、無効化された処理を中止します。",
  ],
};

export function projectDocumentationNote(name: string, ja: boolean) {
  return documentationNotes[name]?.[ja ? 1 : 0];
}
