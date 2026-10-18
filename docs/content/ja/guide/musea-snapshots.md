<!-- Reviewed translation; source: guide/musea-snapshots.md; scope: complete document -->

<span id="snapshot-の識別子と移行"></span>

# スナップショットの識別子と移行

Musea は、プロジェクト内の Art の相対パス、バリアント、ビューポートの寸法、
デバイスの表示倍率を使って、ベースラインがどの撮影に対応するかを記録します。
同名の `Button.art.vue` を別々に比較でき、ホスト用の再ビルドで絶対パスが
変わっても同じベースラインを使えます。

スナップショットのディレクトリでは、**`identities.json` とベースラインの PNG を一緒に**
保存・転送してください。重複しない PNG 名は維持され、名前が重複する
撮影には、入力から一意に決まる `snapshot-<hash>.png` 形式の名前が割り当てられます。

<span id="既存-baseline-の移行"></span>

## 既存のベースラインを移行する

以前の PNG には、どの撮影に対応するかの記録がありません。Art、バリアント、ビューポート
との対応を確認してから、一度だけ明示的に移行します。

```sh
vp exec musea-vrt --adopt-legacy-snapshots --json
```

ホスト済みギャラリーでは URL も指定します。

```sh
vp exec musea-vrt --gallery-url https://example.com/components/ \
  --adopt-legacy-snapshots --json
```

古いギャラリーは snapshot identity version 1 を出力する Musea で
再ビルドしてください。CLI はビルド元のルートを推測しません。
複数の Art に対応する古い PNG は移行できません。Art ごとに新しい
ベースラインを撮影して確認し、その確認が終わるまで古い PNG を保存してください。

## 個別の承認

`projectRoot` からの相対パスを指定し、`.art.vue` は省略します。

```sh
vp exec musea-vrt approve 'right/Button/*' --gallery-url https://example.com/components/
```

複数の Art が一致する `approve 'Button/*'` は拒否します。パターンなしの
`approve` は失敗中の撮影をすべて承認します。`clean` は削除済み Art と識別子の
対応も残し、バリアント固有のビューポートを使う PNG も維持します。

## ルートと同時実行

識別子のルートは既定で Vite の `root` です。別パッケージの Art を
含める場合は共通のルートを設定します。

```ts
musea({ projectRoot: '../..' })
```

CLI と静的ビルドは同じルートを使います。プロジェクト全体を移動しても
識別子は変わりません。ルートを変更したり、プロジェクト内で Art を移動したりすると
識別子が変わるため、ベースラインを移行して確認する必要があります。

同じスナップショットのディレクトリで同時に実行すると、ロックエラーになります。
異常終了後は VRT プロセスが使っていないことを確認してから
`identities.lock` を削除し、`identities.json` は残してください。
Art と撮影の対応を記録するデータが壊れた場合は、撮影・承認・削除より先に復元してください。
