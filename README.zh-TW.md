# Genshin Uncap

[English](README.md) · [简体中文](README.zh-CN.md) · [繁体中文](README.zh-TW.md) · [日本語](README.ja.md)

又一個使用 Rust 編寫的《原神》FPS 解鎖器

小巧、原生，支援 Windows 與 Linux（Wine / Steam Proton）

## 快速開始

### Windows

從 [Releases](https://github.com/Xarth-Mai/genshin-uncap/releases) 下載最新版本

先啟動遊戲，再執行 `genshin-uncap.exe`。程式會自動偵測 `YuanShen.exe` 或 `GenshinImpact.exe`，並將 FPS 上限解鎖至 120

也可以直接透過本程式啟動遊戲：

```bat
genshin-uncap.exe --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120
```

### Linux — Wine / Steam Proton

將 Steam 的啟動目標設定為 `genshin-uncap.exe`，繼續使用遊戲原有的 Proton 版本，並在 Launch Options 中設定：

```text
%command% --game "C:\Games\Genshin Impact Game\YuanShen.exe" --fps 120 --hidden
```

Steam 的 Target 和 Start In 使用 Linux 路徑；`--game` 使用 Wine / Proton prefix 內可見的 Windows 路徑

## F10 暫停 / 恢復

遊戲處於焦點狀態時，按 F10 可以暫停或恢復 FPS 控制

暫停時會恢復本次工作階段開始時的原始 FPS 值；再次按下 F10 後，會重新套用設定的 FPS 上限

## 選項

| 選項 | 說明 |
| --- | --- |
| `--game <path>` | 指定要啟動的遊戲可執行檔；省略時自動偵測或等待 `YuanShen.exe` / `GenshinImpact.exe` |
| `--fps <1..120>` | 設定 FPS 上限，預設 120 |
| `--hidden` | 隱藏控制器視窗，並將記錄寫入 `%LOCALAPPDATA%\genshin-uncap\`；未指定 `--game` 時忽略 |
| `--probe` | 檢查目前遊戲版本是否可識別，但不修改遊戲記憶體 |
| `--help`, `-h` | 顯示說明 |
| `--version`, `-V` | 顯示版本和建置資訊 |
| `-- <arguments...>` | 啟動遊戲時，將後續參數原樣傳遞給遊戲；未指定 `--game` 時忽略 |

## 常見問題

### 如何修改 FPS 上限？

使用 `--fps` 設定所需上限，例如：

```text
--fps 90
```

FPS 上限在一次執行期間保持固定。需要更換數值時，請使用新的 `--fps` 參數重新啟動程式

### 記錄儲存在哪裡？

使用 `--hidden` 時，記錄儲存在：

```text
%LOCALAPPDATA%\genshin-uncap\
```

程式啟動時會自動清理舊記錄，僅保留最新的 10 個控制器記錄

### 為什麼實際 FPS 沒有達到設定值？

`--fps` 設定的是 FPS 上限，並不保證遊戲能夠達到對應幀率

實際幀率仍取決於 GPU / CPU 效能、圖形設定、VSync，以及其他可能存在的限幀機制

### 遊戲更新後無法解鎖 FPS 怎麼辦？

遊戲更新可能導致用於定位 FPS 上限的資料發生變化

回報問題時，請保留主控台中的錯誤訊息；如果使用了 `--hidden`，請同時附上 `%LOCALAPPDATA%\genshin-uncap\` 中的相關記錄

### 如何退出？

直接關閉 `genshin-uncap` 即可

如果希望在退出前恢復本次工作階段原本的 FPS 值，請先按 F10 暫停 FPS 控制，再關閉程式

遊戲退出後，控制器也會自動結束

### 測試過哪些環境？

目前主要測試環境為透過 Steam Proton 執行的 `YuanShen.exe`，包括 XWayland 和原生 Wayland

程式發布為 Windows x64 可執行檔，在 Linux 下透過 Wine / Proton 執行

## 從原始碼建置

```sh
rustup target add x86_64-pc-windows-gnu
cargo build --release --locked
```

輸出檔案位於：

```text
target/x86_64-pc-windows-gnu/release/genshin-uncap.exe
```

在 Linux 上執行邏輯測試：

```sh
cargo test --target x86_64-unknown-linux-gnu --locked
```

## 致謝

感謝以下專案、工具與生態提供的參考、基礎設施、執行環境與開發協助：

* [xiaonian233/genshin-fps-unlock](https://github.com/xiaonian233/genshin-fps-unlock)
* [34736384/genshin-fps-unlock](https://github.com/34736384/genshin-fps-unlock)
* [windows-rs](https://github.com/microsoft/windows-rs)
* [Rust](https://www.rust-lang.org/)
* [Steam](https://store.steampowered.com/)
* [Proton](https://github.com/ValveSoftware/Proton)
* [Wine](https://www.winehq.org/)
* [Linux](https://www.linux.org/)
* OpenAI ChatGPT / Codex

## 開放原始碼授權

本專案採用 [Mozilla Public License 2.0](LICENSE)

第三方軟體聲明見 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)
