# Snapshot の識別子と移行

Musea はプロジェクト内の Art の相対パス、variant、viewport の寸法と
device scale を使って baseline の所有者を記録します。同名の
`Button.art.vue` を別々に比較でき、ホスト用の再ビルドで絶対パスが
変わっても同じ baseline を使えます。

snapshot ディレクトリの **`identities.json` と baseline PNG を一緒に**
保存・転送してください。衝突しない PNG 名は維持され、衝突する
撮影には決定的な `snapshot-<hash>.png` 名が割り当てられます。

## 既存 baseline の移行

以前の PNG には所有者の記録がありません。Art、variant、viewport
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
複数の Art に対応する古い PNG は移行を拒否します。新しい個別
baseline を撮影・確認し、その確認が終わるまで古い PNG を保存します。

## 個別の承認

projectRoot からの相対パスを指定し、`.art.vue` は省略します。

```sh
vp exec musea-vrt approve 'right/Button/*' --gallery-url https://example.com/components/
```

複数の Art が一致する `approve 'Button/*'` は拒否します。パターンなしの
`approve` は失敗中の撮影をすべて承認します。`clean` は削除済み Art の
所有者予約も残し、variant 固有の viewport を使う PNG も維持します。

## ルートと同時実行

識別子のルートは既定で Vite の `root` です。別パッケージの Art を
含める場合は共通のルートを設定します。

```ts
musea({ projectRoot: '../..' })
```

CLI と static build は同じルートを使います。プロジェクト全体の移動は
識別子を維持しますが、ルート変更やプロジェクト内での Art の移動は
識別子が変わるため baseline の移行確認が必要です。

同じ snapshot ディレクトリでの同時実行は lock エラーになります。
異常終了後は VRT プロセスが使っていないことを確認してから
`identities.lock` を削除し、`identities.json` は残してください。
所有者データが壊れた場合は、撮影・承認・削除より先に復元します。
