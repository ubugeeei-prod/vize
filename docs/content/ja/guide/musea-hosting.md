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

スクリーンショットの比較とベースラインの更新には Node と Playwright が必要です。
プロジェクトで `vp exec musea-vrt` を実行するか、開発ギャラリーの **Run VRT** を使います。
静的ホストの VRT パネルにも、この実行方法を表示します。

