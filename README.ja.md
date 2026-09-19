# Genshin Uncap

[English](README.md) · [简体中文](README.zh-CN.md) · [繁体中文](README.zh-TW.md) · [日本語](README.ja.md)

Rust で書かれた、もうひとつの『原神』FPS 解放ツール

小型でネイティブ対応。Windows と Linux（Wine / Steam Proton）をサポートします

## クイックスタート

### Windows

[Releases](https://github.com/Xarth-Mai/genshin-uncap/releases) から最新版をダウンロードしてください

先にゲームを起動してから `genshin-uncap.exe` を実行します。`YuanShen.exe` または `GenshinImpact.exe` を自動検出し、FPS 上限を 120 に解放します

このプログラムからゲームを直接起動することもできます：

```bat
genshin-uncap.exe --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120
```

### Linux — Wine / Steam Proton

Steam の起動対象を `genshin-uncap.exe` に設定し、ゲームで使用している Proton のバージョンをそのまま使います。Launch Options に次を設定してください：

```text
%command% --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120 --hidden
```

Steam の Target と Start In には Linux パスを指定します。`--game` には Wine / Proton prefix 内から見える Windows パスを指定します

## F10 で一時停止 / 再開

ゲームがフォーカスされている状態で F10 を押すと、FPS 制御を一時停止または再開できます

一時停止すると、セッション開始時の元の FPS 値に戻ります。もう一度 F10 を押すと、設定した FPS 上限を再適用します

## オプション

| オプション | 説明 |
| --- | --- |
| `--game <path>` | 起動するゲーム実行ファイルを指定します。省略時は `YuanShen.exe` / `GenshinImpact.exe` を自動検出または待機します |
| `--fps <1..120>` | FPS 上限。デフォルトは 120 |
| `--hidden` | コントローラーのウィンドウを隠し、ログを `%LOCALAPPDATA%\genshin-uncap\` に保存します。`--game` なしでは無視されます |
| `--probe` | 現在のゲームバージョンを認識できるか確認します。ゲームメモリは変更しません |
| `--help`, `-h` | ヘルプを表示します |
| `--version`, `-V` | バージョンとビルド情報を表示します |
| `-- <arguments...>` | ゲーム起動時、後続の引数をそのままゲームへ渡します。`--game` なしでは無視されます |

## FAQ

### FPS 上限を変更するには？

`--fps` で上限を指定します。例：

```text
--fps 90
```

FPS 上限は 1 回の実行中は固定されます。変更する場合は、新しい `--fps` を指定してプログラムを再起動してください

### ログはどこに保存されますか？

`--hidden` 使用時のログは次に保存されます：

```text
%LOCALAPPDATA%\genshin-uncap\
```

起動時に古いログを自動的に削除し、最新のコントローラーログ 10 件だけを保持します

### 実際の FPS が設定値に届かないのはなぜですか？

`--fps` は FPS の上限を設定するもので、ゲームがそのフレームレートに到達することを保証するものではありません

実際の FPS は GPU / CPU 性能、グラフィック設定、VSync、その他の制限機構に左右されます

### ゲーム更新後に FPS を解放できなくなった場合は？

ゲームの更新により、FPS 上限の位置を特定するためのデータが変わることがあります

問題を報告する際は、コンソールのエラーメッセージを保存してください。`--hidden` を使用した場合は、`%LOCALAPPDATA%\genshin-uncap\` 内の関連ログも添付してください

### 終了するには？

`genshin-uncap` を閉じるだけで終了できます

終了前にセッション開始時の FPS 値へ戻したい場合は、先に F10 で FPS 制御を一時停止してからプログラムを閉じてください

ゲーム終了後、コントローラーも自動的に終了します

### どの環境でテストされていますか？

現在の主なテスト環境は Steam Proton 経由で実行する `YuanShen.exe` で、XWayland とネイティブ Wayland を含みます

プログラムは Windows x64 実行ファイルとして配布され、Linux では Wine / Proton 経由で実行します

## ソースからビルド

Windows x64 GNU target を追加します：

```sh
rustup target add x86_64-pc-windows-gnu
```

Release 版をビルドします：

```sh
cargo build --release --locked
```

出力先：

```text
target/x86_64-pc-windows-gnu/release/genshin-uncap.exe
```

Linux 上でロジックテストを実行するには：

```sh
cargo test --target x86_64-unknown-linux-gnu --locked
```

## 謝辞

以下のプロジェクト、ツール、エコシステムから参考、基盤、実行環境、開発支援を受けています：

* [xiaonian233/genshin-fps-unlock](https://github.com/xiaonian233/genshin-fps-unlock)
* [34736384/genshin-fps-unlock](https://github.com/34736384/genshin-fps-unlock)
* [windows-rs](https://github.com/microsoft/windows-rs)
* [Rust](https://www.rust-lang.org/)
* [Steam](https://store.steampowered.com/)
* [Proton](https://github.com/ValveSoftware/Proton)
* [Wine](https://www.winehq.org/)
* [Linux](https://www.linux.org/)
* OpenAI ChatGPT / Codex

## オープンソースライセンス

本プロジェクトは [Mozilla Public License 2.0](LICENSE) を採用しています

第三者ソフトウェアに関する声明は [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) を参照してください
