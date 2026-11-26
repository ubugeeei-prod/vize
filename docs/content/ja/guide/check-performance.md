---
title: 型チェックのパフォーマンス
---

<!-- Reviewed translation; source: guide/check-performance.md -->

<span id="type-checking-performance"></span>

# 型チェックのパフォーマンス

型チェックが遅い場合や、メモリを使いすぎる場合に使うガイドです。TypeScript 7.1 以降では、`tsc` または
[tsrs](https://github.com/oxc-project/tsrs) で動かす[Content Mapper](./content-mapper.md)が推奨の
型チェックです。`vize check` は、TypeScript 7.1 に上げられないプロジェクトのために残しています。まずは
デフォルトのままプロファイルを取り、設定は 1 つずつ変えてください。その他のオプションは
[CLI の型チェック](./cli.md#check)と[型チェックのガイド](./type-checking.md)を参照してください。

## 時間がかかっている場所を調べる

### Content Mapper

Content Mapper のコマンドに `--extendedDiagnostics` を付けます。

```sh
tsrs --runExternalCode --noEmit -p tsconfig.json --extendedDiagnostics
```

TypeScript 7.1 でも同じフラグを使えます。

```sh
tsc --runExternalCode --noEmit -p tsconfig.json --extendedDiagnostics
```

チェックしたファイル、型・シンボル・インスタンス化の数、メモリ使用量、コンパイラーの段階ごとの時間が
表示されます。

### `vize check`

```sh
vpx vize check --profile
```

最後の行に、処理の段階ごとの時間が表示されます。

```diagnostic
✗ Type checked 1 files in 624.27ms (collect: 105.94ms, imports: 0.00ns, gen: 114.51ms, corsa: 339.05ms)
```

| 段階      | 内容                                                                      |
| --------- | ------------------------------------------------------------------------- |
| `collect` | `tsconfig.json` を読み込み、チェックするファイルを探す                    |
| `imports` | コンポーネント間の import をたどる                                        |
| `gen`     | 各 `.vue` ファイルを仮想 TypeScript に変換する                            |
| `corsa`   | 生成したファイルに TypeScript のネイティブコンパイラー（Corsa）を実行する |

プロファイルの表と仮想ファイルは `node_modules/.vize` に書き込まれます。`corsa` が大半を占める場合は、
最後の注記に最も大きい生成ファイルが表示されます。多くの場合、そのコンポーネントを分割・単純化すると
速くなります。

## 遅いプロジェクトを報告する

遅いと感じたら、雑なものでもよいので [Issue](https://github.com/ubugeeei-prod/vize/issues/new) を立ててください。
報告には実際に使っているチェッカーの結果を使います。TypeScript 7.1 か tsrs のプロジェクトなら
Content Mapper の出力、それ以外なら `vize check --profile` の出力です。次の情報を含めてください。

- 実行したコマンドと、`--extendedDiagnostics` または `--profile` の出力全体
- Vize のバージョンと、TypeScript または tsrs のバージョン
- `.vue` ファイルの数と、マシンの OS・CPU コア数・メモリ

## `vize check` を調整する

| 症状                                      | 試すこと                                                                             |
| ----------------------------------------- | ------------------------------------------------------------------------------------ |
| メモリを使いすぎる                        | `--servers 1` で TypeScript のプロセスを 1 つにする                                  |
| 大きなプロジェクトで CPU が余っている     | `--servers` を指定しない。CPU 数とファイル数から自動で決まる                         |
| 1 つの大きなプロジェクトで `corsa` が長い | `--checkers 4` で各プロセス内のチェッカーを増やす（下の注意を参照）                  |
| 編集中に何度も実行する                    | [`check-server`](./type-checking.md#大きなプロジェクトでの速度) を起動したままにする |

```sh
vpx vize check --checkers 4 --servers 1
```

> [!WARNING]
> 1 プロセスあたりのチェッカーを 2 つ以上にすると、診断が出なくなったり変わったりすることがあるため、
> デフォルトは 1 です。CI で `--checkers` を使う前に、デフォルトの出力と比べてください。

`--checkers` は正の整数を受け取り、環境変数 `VIZE_CHECKERS` より優先されます。`--servers`（`1`〜`32`）は
別々に起動する TypeScript プロセスの数で、自動で決める場合は各プロセス内のチェッカー数も考慮します。
すべての実行でサーバー数を固定する場合は `vue.typeChecker.servers` を設定してください。
