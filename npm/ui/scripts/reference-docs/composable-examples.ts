/** Practical SFC examples: one source packet owns the displayed code and live demo. */
import { readFileSync } from "node:fs";
import path from "node:path";
import { previewImageUrl } from "../../../../docs/previews/ui/screenshot-file.ts";
import { COMPOSABLE_CATALOG } from "../../../compose/core/src/catalog.ts";
import { blocks, table } from "./markdown.ts";
import { asyncEffectExamples } from "./async-effect-examples.ts";
import { browserEffectExamples } from "./browser-effect-examples.ts";
import { formStateExamples } from "./form-state-examples.ts";
import { domInteractionExamples } from "./dom-interaction-examples.ts";
import { observedBrowserExamples } from "./observed-browser-examples.ts";
import { scopedInputExamples } from "./scoped-input-examples.ts";
import { preferenceBrowserExamples } from "./preference-browser-examples.ts";
import { environmentExamples } from "./environment-examples.ts";
import { pointerSelectionExamples } from "./pointer-selection-examples.ts";
import { localeFormatExamples } from "./locale-format-examples.ts";
import { workflowStateExamples } from "./workflow-state-examples.ts";
import { timeFormatExamples } from "./time-format-examples.ts";
import { dataStateExamples } from "./data-state-examples.ts";
import { browserPageExamples } from "./browser-page-examples.ts";
import { deviceExamples } from "./device-examples.ts";

const stateExamples = [
  {
    name: "textarea-autosize",
    title: "Write a release note in a growing editor",
    purpose:
      "Keep a multiline draft readable as it grows, without losing the editing position when the available width changes.",
    observe:
      "Type several lines or insert the release template. The editor grows with the text; Compact editor changes the wrapping and the height automatically. Clear draft returns to two rows. Long drafts scroll inside the editor once it reaches 12rem.",
    context:
      "The draft stays in memory and is never saved or submitted. Bind the returned input ref with v-model and pass a template ref for the native textarea. The default height mode allows the example's CSS max-height to cap the editor; min-height is a separate minimum-sizing mode and can override a smaller CSS maximum. Padding and borders are included correctly for either box-sizing value. Programmatic changes are measured after Vue updates the textarea, and real width changes are observed automatically. Server markup keeps the declared rows; measurement and the ResizeObserver belong to the component scope. The package remains experimental.",
  },
  {
    name: "async-resource",
    title: "Request a delivery quote with cancellation",
    purpose:
      "Let a shopper change delivery service or cancel a quote request while keeping the last successful price visible.",
    observe:
      "Start Standard, then immediately Express. The slow Standard loader is aborted and only Express appears. Cancel request stops a pending producer without adding a response. Unavailable area shows a failure while retaining the last quote; choose a service to recover. Reset quote aborts pending work and clears the result.",
    context:
      "This is delayed local sample data, not a server integration: Standard takes 800 ms and the other choices take 300 ms. The producer listens to the real `AbortSignal`, clears its timer on abort and removes its listener when it settles. In an application pass that signal to `fetch`. `keepData` and `cancelPrevious` default to true. `execute()` resolves to a success, error, cancelled or superseded result; a loader rejection does not reject that promise. The response/cancellation counters and last execution label are example-owned history, so reset does not clear them. Call inside setup: no loader runs during SSR, and disposing the component scope aborts active work through the same signal. The package remains experimental.",
  },
  {
    name: "virtual-list",
    title: "Browse a thousand guides",
    purpose:
      "Keep a long, searchable catalogue responsive by rendering only the rows near its visible area.",
    observe:
      "Focus the list and use Page Down, Home or End, or activate Last result. The rendered positions move through 1,000 guides while only a few rows exist at once. Search for 099 to see eleven matches, then try missing for the empty state. Searching returns to the first match without moving focus from the input.",
    context:
      "The complete example uses local sample data, not a server request. Each row must occupy exactly the declared 40 CSS pixels, including padding and borders. `overscan: 2` keeps two additional rows on either side; the container's 200px height determines the visible area. Bind `containerProps` to the scroll region and `wrapperProps.style` to its inner list, then render `list` rather than the entire source. `scrollTo()` takes a zero-based position in the filtered list. The first seven rows render on the server; measurement and scroll tracking belong to the component scope. Only currently rendered rows are available to browser Find and screen readers. Position and total-count attributes describe them, but do not make the other rows available: use search to narrow the catalogue, and prefer pagination or a complete list when every item must be readable or contain focusable controls.",
  },
  {
    name: "use-toggle",
    title: "Show delivery details",
    purpose:
      "Reveal additional information while keeping the disclosure button's accessible state synchronized.",
    observe:
      "Activate the button with Enter or Space. The delivery details appear, the label changes, and `aria-expanded` follows the same boolean ref.",
    context:
      "`state` is writable owned state; `toggle()` inverts it. Calling `toggle(false)` closes it explicitly. No browser globals or cleanup are needed.",
  },
  {
    name: "use-counter",
    title: "Reserve workshop seats",
    purpose:
      "Keep a seat quantity between one and five and explain why further changes are unavailable.",
    observe:
      "Add seats until the limit message appears. Add and remove controls disable at their respective bounds; Reset restores the initial one-seat reservation.",
    context:
      "Mutate the readonly `count` through its controls. `atMin` and `atMax` are computed refs. Bounds are fixed when the composable is created.",
  },
  {
    name: "use-debounced",
    title: "Search the guide catalogue",
    purpose:
      "Update local search results only after typing stops, with explicit pending, flush, cancel, and empty states.",
    observe:
      "Type component. The input changes immediately while the result list waits 500 ms. Search now applies it immediately; Cancel keeps the last settled results. Try a term with no matches.",
    context:
      "`query` is the writable source; `debounced` is its readonly delayed view. This example filters local data and sends no network request. Call inside setup so its watcher and timer belong to the component scope. SSR mirrors the source without starting a timer.",
  },
  {
    name: "use-field",
    title: "Validate a display name",
    purpose:
      "Validate a profile field on blur and expose visited, changed, error, and reset states.",
    observe:
      "Enter Al and press Tab to see the validation message. Replace it with Ada and leave the field again to clear the error. Reset clears both the value and interaction state.",
    context:
      "Destructure the returned refs before using them in a template. Wire `onBlur` to the native blur event. The error records the last blur; subsequent edits wait for another blur, so an empty recorded error does not promise that an unvalidated edit is ready. The example checks name length, not server availability; no validation runs during setup.",
  },
  {
    name: "use-history",
    title: "Edit and undo a release title",
    purpose:
      "Give a text editor bounded undo/redo and group a multi-write operation into one undo step.",
    observe:
      "Edit the title and try Undo and Redo. Apply publication title performs its two assignments as one step. A new edit after Undo discards redo history; Clear history retains the title.",
    context:
      "Each ref assignment is recorded synchronously. The example keeps at most ten undo entries. Object mutations require reassignment and an appropriate clone function. Its watcher and retained history follow the component scope.",
  },
  {
    name: "use-offset-pagination",
    title: "Browse a guide catalogue",
    purpose:
      "Slice a local list into pages with previous/next boundaries and a writable page-size selector.",
    observe:
      "Go to the last page, then change Guides per page to five. The page is clamped to the new last page and the visible list changes. Previous and Next disable at the boundaries.",
    context:
      "`currentPage` is one-based; `offset` is zero-based. `currentPageSize` can be used with `v-model.number`. The composable manages state; the example's computed ref slices the data.",
  },
] as const;

export const composableExamples = [
  ...stateExamples,
  ...asyncEffectExamples,
  ...browserEffectExamples,
  ...formStateExamples,
  ...domInteractionExamples,
  ...observedBrowserExamples,
  ...scopedInputExamples,
  ...preferenceBrowserExamples,
  ...environmentExamples,
  ...pointerSelectionExamples,
  {
    name: "use-throttled",
    title: "Preview a release title while editing",
    purpose:
      "Keep the latest release-title draft editable while limiting how often its local preview updates.",
    observe:
      "Change the title once to update the preview immediately. Further edits in the same 800 ms window wait; the newest draft appears when that window ends. Update preview now applies the waiting draft immediately. Cancel waiting update keeps both your draft and the last accepted preview; the next edit starts a fresh window.",
    context:
      "The example uses useThrottled's default leading and trailing edges and its real timer. `pending` describes a queued update, not whether the draft equals the preview. `flush()` applies waiting work and closes its window; `cancel()` discards waiting work and closes its window without changing the draft. SSR mirrors the source without starting timers. Call inside setup so scope disposal releases the watcher and timer. This local preview neither saves nor publishes a title; the package remains experimental.",
  },
  ...localeFormatExamples,
  ...workflowStateExamples,
  ...timeFormatExamples,
  ...dataStateExamples,
  ...browserPageExamples,
  ...deviceExamples,
] as const;

/** Reviewed Japanese task labels; shared demo source and API names remain unchanged. */
export const japaneseComposableGoals: Record<(typeof composableExamples)[number]["name"], string> =
  {
    "textarea-autosize": "入力に合わせて広がる欄でリリースノートを書く",
    "async-resource": "配送の見積もりを取得し、中止や再試行を行う",
    "virtual-list": "1,000 件のガイドを検索して一覧をスクロールする",
    "use-toggle": "配送の詳細を開閉する",
    "use-counter": "ワークショップの予約人数を増減する",
    "use-debounced": "入力が落ち着いてからガイドを検索する",
    "use-field": "表示名を入力して検証する",
    "use-history": "リリースのタイトルを編集して元に戻す",
    "use-offset-pagination": "ガイドの一覧をページに分ける",
    "use-interval": "下書きのレビュー時間を計る",
    "use-timeout": "レビューのリマインダーを予約する",
    "use-async-state": "チーム情報を取得し、失敗したら再試行する",
    "use-storage": "ガイドの読書位置を保存する",
    "media-query": "画面の広さに合わせておすすめを変える",
    "active-element": "入力中のプロフィール項目に説明を表示する",
    "use-form": "ワークショップの申込内容を検証する",
    "use-stepper": "申込内容を手順に沿って確認・編集する",
    "use-selection": "項目の識別子を保って参加セッションを選ぶ",
    "use-cycle-list": "コンポーネントの確認ポイントを順に表示する",
    focus: "入力欄のフォーカスを確認・移動する",
    "on-click-outside": "設定の外側をクリックして閉じる",
    "on-key-stroke": "選んだ操作部品でショートカットを受け付ける",
    "event-listener": "選んだ下書きの入力イベントを監視する",
    "use-clipboard": "編集したインストールコマンドをコピー・読み取りする",
    scroll: "準備ガイドの表示位置をスクロールする",
    "resize-observer": "選んだパネルのサイズを監視する",
    "intersection-observer": "スクロール領域内でカードが見えるか確認する",
    "element-bounding": "ガイドのパネルの位置と大きさを測る",
    "element-hover": "カードにポインターを置いてから説明を表示する",
    "on-long-press": "ボタンの長押し中にレビューのメモを表示する",
    "on-start-typing": "文字入力を始めたらガイドの検索欄に移動する",
    network: "オフラインでもガイドの下書きを編集する",
    "color-mode": "配色を選ぶ、またはシステムの設定に合わせる",
    breakpoints: "画面幅に合わせてガイドの詳しさを変える",
    "window-size": "画面の大きさに合わせてカードを並べる",
    pointer: "操作領域のポインターイベントを確認する",
    "mouse-in-element": "画面内と要素内のマウス座標を比較する",
    draggable: "決めた範囲内でメモをドラッグする",
    "text-selection": "編集できる文章の選択範囲を確認する",
    "use-throttled": "タイトルの編集中に一定間隔でプレビューを更新する",
    "use-number-format": "読み手の地域に合う形式で注文金額を表示する",
    "use-list-format": "登壇者の名前を自然な列挙形式で表示する",
    "use-plural-rules": "未読件数に合わせて単語の形を選ぶ",
    "use-machine": "条件を確認しながら購入手続きを進める",
    "use-previous": "チケット価格の前回からの変化を表示する",
    "use-countdown": "ログインコードを再送できるまでの時間を表示する",
    "use-relative-time-format": "イベントの開催時刻を現在からの相対表現で示す",
    "use-array": "持ち物リストにチェックを付ける",
    "use-set": "フォローする話題を選ぶ",
    "use-map": "カート内の商品の個数を管理する",
    "use-queue": "文書を追加した順に印刷する",
    "use-stack": "前の画面に戻るための履歴を管理する",
    "use-sorted": "スコアを順位順に並べる",
    "use-sorted-locale": "言語に合う順序で名前を並べる",
    "use-memoize": "計算済みの送料を再利用する",
    "use-id-generator": "繰り返すフォーム項目に識別子を付ける",
    "use-date-format": "指定したタイムゾーンで開始時刻を表示する",
    "use-date-time-format": "読み手の地域に合う形式で日付を表示する",
    "use-display-names": "言語の名前を表示する",
    "use-segmenter": "投稿文の文字数を数える",
    "use-now": "現在時刻を更新して表示する",
    "use-time-ago": "下書きを保存してからの経過時間を表示する",
    "use-supported": "ブラウザーが機能に対応しているか確認する",
    "use-mounted": "マウント後にブラウザー専用の内容を表示する",
    "use-confirm-dialog": "下書きの削除前に確認する",
    "use-async-queue": "依存する処理を順に実行する",
    "use-v-model": "v-model と内部の状態を使い分ける",
    "use-form-field-props": "入力欄をラベルやエラー表示と結び付ける",
    "use-route-query": "選択したタブを URL のクエリーに残す",
    "use-url-hash": "URL のハッシュでページ内の節に移動する",
    "use-url-search-params": "検索条件をアドレスバーに残す",
    "use-cookie": "配色の設定を Cookie に保存する",
    "use-indexed-db": "下書きを IndexedDB に保存する",
    "use-css-var": "CSS 変数でカードの見た目を変える",
    "use-document-title": "ブラウザーのタブのタイトルに未読件数を表示する",
    "use-favicon": "タブのアイコンでビルドの状態を伝える",
    "use-style-tag": "機能を使っている間だけ CSS を追加する",
    "use-script-tag": "必要になったら外部のウィジェットを読み込む",
    "use-object-url": "入力した文章をファイルとしてダウンロードする",
    "use-image": "画像を取得し、失敗したら代わりの表示を出す",
    "use-fetch": "操作に応じて更新情報を取得する",
    "use-event-source": "サーバーから届くイベントを受け取る",
    "use-websocket": "WebSocket でチャットに接続する",
    "use-broadcast-channel": "別のタブに下書きの変更を知らせる",
    "use-worker-fn": "重い計算を Web Worker で実行する",
    "use-compression-stream": "アップロード前に文章を圧縮する",
    "use-raf-fn": "進捗バーをアニメーションで動かす",
    "use-request-idle-callback": "ブラウザーの空き時間に分析データを送る",
    "use-scheduler-post-task": "優先度に応じて処理を実行する",
    "use-web-locks": "書き出し処理を一度に一つだけ実行する",
    "use-view-transition": "レイアウトの切り替えをアニメーションで示す",
    "use-performance-observer": "ページのリクエストにかかった時間を測る",
    "use-web-vitals": "Core Web Vitals の計測値を表示する",
    "use-storage-estimate": "ストレージの使用量を確認する",
    "use-visual-viewport": "モバイルで実際に見えている領域を調べる",
    "use-user-activation": "ユーザーが最初に操作したことを確認する",
    "use-navigation-api": "ウィザードの手順をブラウザーの履歴に残す",
    "use-drop-zone": "ドロップした画像を受け付ける",
    "use-file-dialog": "用意したボタンからファイルを選ぶ",
    "use-media-controls": "音声の再生コントロールを作る",
    "use-bluetooth": "心拍計とペアリングする",
    "use-hid": "HID デバイスに接続する",
    "use-serial": "マイコンのログを読み取る",
    "use-usb": "USB デバイスへのアクセスを許可する",
    "use-web-midi": "MIDI コントローラーの一覧を表示する",
    "use-gamepad": "接続されたゲームコントローラーを表示する",
    "use-payment-request": "ブラウザーの支払い画面を開く",
    "use-web-authn": "パスキーでのログインを用意する",
    "use-contact-picker": "連絡先から招待する相手を選ぶ",
    "use-eye-dropper": "画面上の色を取得する",
    "use-barcode-detector": "写真に写った QR コードを読み取る",
    "use-file-system-access": "ローカルのファイルを直接編集する",
    "use-fullscreen": "スライドを全画面で表示する",
    "use-notification": "トークの開始前に通知する",
    "use-permission": "権限の状態に応じて表示を変える",
    "use-share": "リンクを共有する",
    "use-vibrate": "振動で操作への反応を伝える",
    "use-wake-lock": "画面が自動で消えないようにする",
    "use-screen-details": "接続された別の画面を探す",
    "use-display-media": "画面を共有する",
    "use-user-media": "カメラの映像をプレビューする",
    "use-speech-recognition": "音声で検索語を入力する",
    "use-speech-synthesis": "案内を音声で読み上げる",
    "use-service-worker": "オフラインで使う準備をする",
    "use-push-subscription": "予定の通知を購読する",
    "use-builtin-ai": "端末内でコメントの言語を判定する",
  };

export function composableExampleSource(packageRoot: string, name: string): string {
  if (!COMPOSABLE_CATALOG.entries.some((entry) => entry.subpath === `./${name}`))
    throw new Error(`Example has no public composable entry: ${name}`);
  return readFileSync(path.join(packageRoot, "examples", `${name}.vue`), "utf8");
}

export function composablePreviewMarkup(name: string, title: string): string {
  const url = `/component-previews/app/index.html?composable=${encodeURIComponent(name)}`;
  return blocks(
    `<iframe src="${url}" title="${title} interactive example" loading="lazy" width="100%" height="540" style="border:1px solid #8885;border-radius:8px"></iframe>`,
    `[Open the interactive example](${url})`,
  );
}

export function composableExampleSection(packageRoot: string, name: string): string {
  const example = composableExamples.find((item) => item.name === name);
  if (example == null) return "";
  return blocks(
    "## Try it",
    `### ${example.title}`,
    example.purpose,
    composablePreviewMarkup(name, example.title),
    `**What to observe:** ${example.observe}`,
    example.context,
    `<details><summary>Compare the initial and interacted states</summary><div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(min(260px,100%),1fr));gap:16px"><figure style="margin:0"><img src="${previewImageUrl(`composables/${name}-720-initial.png`)}" alt="${example.title}: initial state" loading="lazy" style="width:100%" /><figcaption>Initial state</figcaption></figure><figure style="margin:0"><img src="${previewImageUrl(`composables/${name}-720-interacted.png`)}" alt="${example.title}: state after the verified interactions" loading="lazy" style="width:100%" /><figcaption>After the verified interactions</figcaption></figure></div></details>`,
    name === "use-storage"
      ? `<details><summary>When browser storage is unavailable</summary><p>Choices still change in this tab. Reloading restores Getting started because the browser cannot save the choice. This capture uses actual Chromium with localStorage disabled.</p><img src="${previewImageUrl("composables/use-storage-unavailable-360-interacted.png")}" alt="Publishing selected in this tab while browser storage is unavailable" loading="lazy" width="360" style="max-width:100%" /></details>`
      : "",
    "## Copy the example",
    "Save this complete SFC in your Vue project. Its public imports and source are the same ones executed in the preview.",
    ["```vue", composableExampleSource(packageRoot, name).trim(), "```"].join("\n"),
  );
}

export function composableHub(locale: string, linkPrefix: string): string {
  const ja = locale === "ja";
  return blocks(
    ja ? "## 動作から選ぶ" : "## Try a behavior",
    ja
      ? "作りたい動作から使用例を選べます。リンク先の API 説明と操作画面は英語です。各ページに、操作できるプレビュー、確認する状態の変化、そのままコピーできる Vue SFC を掲載しています。プレビューと掲載コードには同じ SFC を使っています。"
      : "See how reactive state changes an interface. Each live preview and its copyable code come from the same complete Vue SFC.",
    table(
      [ja ? "やりたいこと" : "Goal", "Composable"],
      composableExamples.map((example) => [
        `[${ja ? japaneseComposableGoals[example.name] : example.title}](${linkPrefix}${example.name}.md)`,
        `\`${example.name}\``,
      ]),
    ),
    ja ? "## 最小構成" : "## Minimal setup",
    "```bash\nvp install @vizejs/composable\n```",
    ja
      ? "Vue 3.5 以降のプロジェクトにインストールし、使用例のページにある SFC 全体をコピーしてください。監視・イベント購読・タイマーを持つ関数は `<script setup>` 内で呼び出します。コンポーネントが破棄されると、対応する監視や購読、タイマーも解除されます。"
      : "Use a Vue 3.5+ project, then copy the complete SFC from an example page. Call lifecycle-bound utilities inside `<script setup>` so their watchers, listeners, and timers are disposed with the component.",
    ja
      ? "カメラや Bluetooth などの例は、ブラウザーの対応状況、接続した機器、権限によって試せる操作が変わります。プレビューで対応していない機能の表示を確認できても、実機での接続や許可後の動作まで検証済みとは限りません。各ページの対応範囲と、実行環境を確認してください。"
      : "",
    ja ? "## 全 API" : "## All APIs",
    ja
      ? `以下は全公開エントリーのソース由来リファレンスです。上記の ${composableExamples.length} 例以外については、ライブプレビューでの検証をまだ行っていません。`
      : `The source-generated reference below covers every public entry. Live preview interaction checks currently cover the ${composableExamples.length} examples above; other entries retain their API examples and runtime contracts.`,
  );
}
