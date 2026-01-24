## Q.Cookieを読み込んでダウンロードしようとするとエラーが発生します

A. denoがインストールされていない可能性が高いです。  
denoは次の方法でインストールが可能です。

> [!NOTE]
> `volta`や`proto`などのパッケージ管理システムには対応していません。

### Windows

```bash
irm https://deno.land/install.ps1 | iex
```

### Linux, macOS

```bash
curl -fsSL https://deno.land/install.sh | sh
```

## Q.動画が結合されません

A. ffmpegが未インストールの可能性があります。  
以下の方法でインストールが可能です。

### Windows

```bash
winget install Gyan.FFmpeg
```

### macOS

```bash
brew install ffmpeg
```

### Linux

```bash
wget https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz
tar -xf ffmpeg-master-latest-linux64-gpl.tar.xz
sudo cp ./ffmpeg-master-latest-linux64-gpl/bin/* /usr/local/bin/
```

## Q.ChromeからCookieを読み込めないのはなぜですか？

A. Chromeやその他Chromiumベースのブラウザはその仕組みからFirefox系ほど簡単にcookieを取得できません。  
またcookies.txtを読み込む方法は現在非推奨になっているため、サポートすることはありません。

Cookieが必要であればFirefoxベースのブラウザをお使いください。

## Q.macOSでyt-dlpが実行されるまで時間がかかります。

A. 仕様です。公式に配布されているバイナリがx86_64向けなのでしょう。