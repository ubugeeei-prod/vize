# Musea のホスティング

`vize` npm パッケージをインストールすると、`vp exec vize musea` は Vite の便利なラッパーになります。

```bash
vp exec vize musea
vp exec vize musea --build
```

ビルドしたギャラリーでもプレビューの iframe でアクセシビリティテストを実行できます。
ビルド前に `axe-core` をインストールすると、テスト用のバンドルも出力されます。
非同期の `previewSetup` にも対応し、プレビューのマウントを待って実行します。
読み込みやテストの実行エラーは失敗として表示されます。`vendor` ディレクトリを含む
出力全体を、設定した Vite の base パスでホストしてください。

## ホストしたギャラリーの VRT

CLI を実行する環境に Playwright と Chromium をインストールします。

```bash
vp add -D playwright
vp exec playwright install chromium
vp exec musea-vrt --gallery-url https://example.com/site/__musea__/ --json --ci
```

静的ギャラリーの VRT パネルにも、そのギャラリーの URL を使うコマンドを表示します。
CLI は `api/static.json` を読み、出力された各プレビューの URL を撮影します。
元の Art ソースやホスト側の Node サービスは不要です。プレビュー、アセット、manifest
を含むビルド出力全体を指定した URL で公開してください。

初回は `.vize/snapshots` にベースラインを作成し、以降はその画像と比較します。
非同期 `previewSetup` とマウントの完了を待って撮影します。画像の差分や撮影エラーが
あると `--ci` は失敗終了します。JSON レポートにはベースライン、現在の画像、差分の
パスと変更ピクセル数を含みます。`--output path` で保存先を変更できます。

現在の画像と差分を確認してから変更を承認します。

```bash
vp exec musea-vrt approve --gallery-url https://example.com/site/__musea__/
vp exec musea-vrt clean --gallery-url https://example.com/site/__musea__/
```

`approve` は現在のギャラリーを再撮影し、失敗したベースラインを更新します。
`clean` は manifest から削除されたバリアントのベースラインを削除します。
保存先や撮影条件を変更する場合は、各コマンドに同じ `--config` と `--output` を渡してください。

開発サーバーでは、プロジェクトで `vp exec musea-vrt` を実行するか **Run VRT** を使います。
CLI は Vite の base と Musea の `basePath` を反映します。サーバーの origin は
`--base-url` で、静的ホストのギャラリーは `--gallery-url` で指定します。
