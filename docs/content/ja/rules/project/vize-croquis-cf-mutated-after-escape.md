---
title: "vize:croquis/cf/mutated-after-escape"
---

# `vize:croquis/cf/mutated-after-escape`

reactive オブジェクトが、所有者から漏れた後に変更されています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI の診断は、重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

変更しない履歴という明示的な所有方針を示す例です。リアクティブなオブジェクトを渡したり、後で変更したりすることを一般に禁止するものではありません。現在この契約の生成元はありません。

## 共通のプロジェクト ファイル

以下のファイルは、悪い例・良い例の両方で変更せずに使います。プロジェクトには Vue をインストールし、例で import している場合は vue-router や Pinia もインストールしてください。バージョン固有の対応条件がある場合は、その注意に従ってください。エントリーファイルによって、コンポーネント間の関係を明示しています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`archive.ts`

```ts
export interface Profile { name: string }
const records: Readonly<Profile>[] = [];
export function publish(profile: Readonly<Profile>): void { records.push(profile); }
export function latestName(): string { return records.at(-1)?.name ?? ''; }

```

`App.vue`

```vue
<script setup lang="ts">
import { publishProfile } from './profile';
import { latestName } from './archive';
publishProfile();
const archivedName = latestName();
</script>

<template>
<p>Archived name: {{ archivedName }}</p>
</template>

```

## 悪い

archive は `publish` に渡した同じオブジェクトを保持します。その後で所有側が名前を変更し、過去の記録まで Grace に変わります。TypeScript の Readonly 引数はオブジェクトをコピーしません。

`profile.ts`

```ts
import { reactive } from 'vue';
import { publish } from './archive';
export function publishProfile(): void {
  const profile = reactive({ name: 'Ada' });
  publish(profile);
  profile.name = 'Grace';
}

```

## 良い

通常のコピーを公開し、保存した Ada の記録と後のリアクティブな profile の変更を分離します。archive のスナップショット方針を保てるようになります。

`profile.ts`

```ts
import { reactive } from 'vue';
import { publish } from './archive';
export function publishProfile(): void {
  const profile = reactive({ name: 'Ada' });
  publish({ ...profile });
  profile.name = 'Grace';
}

```

良い例のファイルは、上で説明した変更を示すものです。プロジェクト全体を検査すると、ほかの診断が出る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
