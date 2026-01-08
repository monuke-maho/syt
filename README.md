# syt

<img width="1172" height="727" alt="image" src="https://github.com/user-attachments/assets/befcd9b6-e6b3-4bf6-abc3-53ba6d1bb1bf" />

yt-dlpをGUIで使えるようにするアプリケーション

## 現状

- [x] 基本機能の最低限の実装
- [x] 使いやすいUIの構築
- [ ] 基本機能の充実
- [ ] 高度な機能の実装

## 推奨動作環境

### OS

本アプリケーションはWindows向けに開発されており、動作確認もWindowsで行なっています。  
macOSでも開発・確認を行ってはいますが、すべての機能が動作する保証はありません。

### 依存関係

- **ffmpeg**  
  `yt-dlp`は動画を変換・統合する際に`ffmpeg`を使用します。
  Windowsの場合は以下のコマンドでインストールが可能です。
  ```shell
  winget install -e --id Gyan.FFmpeg
  ```
  macOSの場合はhomebrewを使用してインストールが可能です。
  ```shell
  brew install ffmpeg
  ```

## 開発環境

- Windows 11 Pro (25H2)
- Google Antigravity or Visual Studio Code
