## 依存関係

> [!CAUTION]
> **絶対に**インストールしてください。

### deno

> [!NOTE]
> `volta`や`proto`などのパッケージ管理システムには対応していません。

#### Windows

```bash
irm https://deno.land/install.ps1 | iex
```

#### Linux, macOS

```bash
curl -fsSL https://deno.land/install.sh | sh
```

### ffmpeg

#### Windows

```bash
winget install Gyan.FFmpeg
```

#### macOS

```bash
brew install ffmpeg
```

#### Linux

```bash
wget https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz
tar -xf ffmpeg-master-latest-linux64-gpl.tar.xz
sudo cp ./ffmpeg-master-latest-linux64-gpl/bin/* /usr/local/bin/
```