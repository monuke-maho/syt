<script setup lang="ts">
// import
import { ref, reactive, onMounted, nextTick, watch, computed } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { homeDir } from '@tauri-apps/api/path';
import { Command } from '@tauri-apps/plugin-shell';
import { platform } from '@tauri-apps/plugin-os';
import { LazyStore } from '@tauri-apps/plugin-store';

import { Button } from './components/ui/button';
import { Input } from './components/ui/input';
import { Progress } from './components/ui/progress';
import { Switch } from './components/ui/switch';
import { Label } from './components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue, SelectLabel } from './components/ui/select';

interface QualityItem {
  label: string;
  value: string;
}

interface AppSettings {
  savePath: string;
  selectedExt: string;
  selectedQuality: string;
  playlistMode: boolean;
  namedIndex: boolean;
  embedThumbnails: boolean;
  cropdThumbnails: boolean;
  embedMetadata: boolean;
}

const SETTINGS_KEY = 'app-config';
const store = new LazyStore('settings.json')
const currentOS = platform()

const EXTS = ['mp4', 'mkv', 'mp3', 'flac', 'wav']
const VIDEO_QUALITYS: QualityItem[] = [
  { label: '自動', value: 'auto' },
  { label: '4K', value: '2160' },
  { label: '2K', value: '1440' },
  { label: '1080p', value: '1080' },
  { label: '720p', value: '720' }
]
const AUDIO_QUALITIES: QualityItem[] = [
  { label: '自動', value: 'auto' },
  { label: '320Kbps', value: '320' },
  { label: '256Kbps', value: '256' },
  { label: '192Kbps', value: '192' },
  { label: '128Kbps', value: '128' }
];

const isInit = ref(false)
const settings = reactive<AppSettings>({
  savePath: '',
  selectedExt: 'mp4',
  selectedQuality: 'auto',
  playlistMode: false,
  namedIndex: false,
  embedThumbnails: false,
  cropdThumbnails: false,
  embedMetadata: false
})

const videoUrl = ref('');
const downloading = ref(false);
const downloadProgress = ref<number | null>(0);
const downloadTitle = ref('');
const downloadLog = ref<string[]>([]);
const downloadErrors = ref<string[]>([]);
const logArea = ref<HTMLElement | null>(null);

const qualityOptions = computed(() => {
  if (settings.selectedExt === 'mp3') {
    return AUDIO_QUALITIES;
  } else if (['mp4', 'mkv'].includes(settings.selectedExt)) {
    return VIDEO_QUALITYS;
  } else {
    return [];
  }
});

const addLog = (msg: string) => downloadLog.value.push(msg);

const saveToDisk = async () => {
  if (!isInit.value || !store) return;
  try {
    await store.set(SETTINGS_KEY, { ...settings });
    await store.save();
    console.log('SAVED!!')
  } catch (err) {
    console.error('ERROR:', err)
  }
}

watch(settings, saveToDisk, { deep: true })

// ログの自動スクロール
watch(downloadLog, async () => {
  await nextTick();
  if (logArea.value) {
    logArea.value.scrollTop = logArea.value.scrollHeight;
  }
}, { deep: true })

// 起動時の処理
onMounted(async () => {
  try {
    await store.init();
    await store.reload();

    const saved = await store.get<AppSettings>(SETTINGS_KEY);
    const defaultPath = await homeDir();

    if (saved) {
      Object.assign(settings, saved);
    } else {
      settings.savePath = defaultPath;
    }

    await nextTick();
    isInit.value = true;

    addLog('[🚀] yt-dlpの更新を確認中...')
    const updateCmd = Command.sidecar('binaries/yt-dlp', ['-U']);
    updateCmd.stdout.on('data', (line) => addLog(`[UPDATE] ${line.trim()}`))
    await updateCmd.spawn()
  } catch (err) {
    console.error('ERROR: ', err)
  }
});

// 保存先選択処理
const selectSaveDir = async () => {
  const dir = await open({ multiple: false, directory: true });
  if (dir && typeof dir === 'string') {
    settings.savePath = dir;
  }
}

// ダウンロード処理
const downloadVideo = async () => {
  if (!videoUrl.value) return addLog('[❌] URLが入力されていません')

  downloading.value = true;
  downloadProgress.value = 0;
  downloadTitle.value = '';
  downloadErrors.value = [];
  addLog('[⬇️] ダウンロードを開始します...')

  const isAudio = ['mp3', 'flac', 'wav'].includes(settings.selectedExt);
  const encoding = currentOS === 'windows' ? 'shift_jis' : 'utf-8';
  const env: Record<string, string> = currentOS === 'macos' ? { PATH: '/opt/homebrew/bin:/usr/local/bin:/usr/bin:/usr/sbin:/sbin' } : {}
  const ytdlopts = ['--newline', '--no-color', '--progress-template', '[DOWNLOADING]::%(progress._percent)s::%(info.title)s']

  if (!isAudio) {
    const q = settings.selectedQuality;
    const format = q === 'auto' ? 'bestvideo+bestaudio[ext=m4a]/best' : `bestvideo[height<=${q}]+bestaudio[ext=m4a]/best[height<=${q}]`;
    ytdlopts.push('-f', format, '--merge-output-format', settings.selectedExt)
  } else {
    ytdlopts.push('-f', 'bestaudio/best', '-x', '--audio-format', settings.selectedExt);
    if (settings.selectedExt === 'mp3') {
      const q = settings.selectedQuality === 'auto' ? '0' : settings.selectedQuality;
      ytdlopts.push('--audio-quality', q)
    }
  }

  let outputTemplate = settings.savePath;
  if (settings.playlistMode) {
    outputTemplate += settings.namedIndex ? '/%(playlist_title)s/%(playlist_index)s - %(title)s.%(ext)s' : '/%(playlist_title)s/%(title)s.%(ext)s'
  } else {
    outputTemplate += '/%(title)s.%(ext)s';
  }

  if (settings.embedThumbnails && settings.selectedExt !== 'wav') {
    ytdlopts.push('--embed-thumbnail')
    if (settings.cropdThumbnails) {
      ytdlopts.push("--convert-thumbnails", "jpg", "--ppa", "ThumbnailsConvertor:-qmin 1 -q:v 1 -vf crop=\"'if(gt(ih,iw),iw,ih)':'if(gt(iw,ih),ih,iw)'\"")
    }
  }

  if (settings.embedMetadata && settings.selectedExt !== 'wav') {
    ytdlopts.push('--add-metadata')
  }

  ytdlopts.push('-o', outputTemplate, videoUrl.value)

  console.log('yt-dlp options:', ytdlopts)
  const cmd = Command.sidecar('binaries/yt-dlp', ytdlopts, { encoding: encoding, env: env })
  cmd.stdout.on('data', (line: string) => {
    const trimmed = line.trim()
    if (trimmed.startsWith('[DOWNLOADING]')) {
      const [, percent, title] = trimmed.split('::');
      downloadProgress.value = parseFloat(percent)
      if (title) downloadTitle.value = title
    } else {
      addLog(trimmed)
      downloadProgress.value = null
    }
  })
  cmd.stderr.on('data', (line: string) => {
    downloadErrors.value.push(line.trim())
  })
  await cmd.spawn()
  cmd.on('close', (data) => {
    downloading.value = false
    downloadTitle.value = ''
    if (data.code === 0) {
      downloadProgress.value = 100
      downloadLog.value.push('[✅] ダウンロードが完了しました！')
    } else if (data.code === 1) {
      downloadProgress.value = 0
      downloadLog.value.push('[❌] ダウンロード中にエラーが発生しました。')
      downloadErrors.value.forEach((err) => {
        downloadLog.value.push('[ERROR] ' + err)
      })
    }
  })
}
</script>

<template>
  <main class="p-6 h-screen min-h-screen relative flex flex-col gap-2">
    <h1 class="text-2xl font-bold">Syt</h1>
    <div class="flex flex-row items-center w-full gap-2">
      <Input v-model="videoUrl" type="text" placeholder="URLを入力..." />
    </div>
    <div class="flex flex-row items-center w-full gap-2">
      <Input v-model="settings.savePath" type="text" placeholder="保存先" readonly />
      <Button @click="selectSaveDir"><span class="material-icons">folder</span>保存先を選択</Button>
    </div>
    <h2 class="text-lg font-bold">オプション</h2>
    <div class="flex flex-col sm:flex-row items-center w-full gap-2">
      <div class="flex flex-row items-center w-full">
        <Label for="extSelect" class="whitespace-nowrap mr-2">拡張子</Label>
        <Select v-model="settings.selectedExt" class="w-32" id="extSelect">
          <SelectTrigger class="w-full">
            <SelectValue placeholder="拡張子を選択" />
          </SelectTrigger>
          <SelectContent>
            <SelectLabel>拡張子を選択</SelectLabel>
            <SelectItem v-for="ext in EXTS" :value="ext" :key="ext">{{ ext }}</SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div class="flex flex-row items-center w-full">
        <Label for="qualitySelect" class="whitespace-nowrap mr-2">品質</Label>
        <Select v-model="settings.selectedQuality" class="w-32" id="qualitySelect">
          <SelectTrigger class="w-full">
            <SelectValue placeholder="品質を選択" />
          </SelectTrigger>
          <SelectContent>
            <SelectLabel>品質を選択</SelectLabel>
            <SelectItem v-for="quality in qualityOptions" :value="quality.value" :key="quality.value">{{ quality.label
              }}</SelectItem>
          </SelectContent>
        </Select>
      </div>
    </div>
    <div class="flex flex-wrap w-full gap-2">
      <div class="flex flex-row gap-2">
        <Switch v-model="settings.playlistMode" id="playlistSwitch" />
        <Label for="playlistSwitch">プレイリストモード</Label>
      </div>
      <div class="flex flex-row gap-2">
        <Switch :disabled="!settings.playlistMode" v-model="settings.namedIndex" id="namedIndexSwitch" />
        <Label for="namedIndexSwitch">インデックスをファイル名に追加する</Label>
      </div>
      <div class="flex flex-row gap-2">
        <Switch v-model="settings.embedThumbnails" id="embedThumbnailsSwitch" />
        <Label for="embedThumbnailsSwitch">サムネイルを埋め込む</Label>
      </div>
      <div class="flex flex-row gap-2">
        <Switch :disabled="!settings.embedThumbnails" v-model="settings.cropdThumbnails" id="cropThumbnailsSwitch" />
        <Label for="cropThumbnailsSwitch">サムネイルを1:1にクロップ</Label>
      </div>
      <div class="flex flex-row gap-2">
        <Switch v-model="settings.embedMetadata" id="embedMetadataSwitch" />
        <Label for="embedMetadataSwitch">メタデータを埋め込む</Label>
      </div>
    </div>
    <h2 class="text-lg font-bold">ログ</h2>
    <div class="flex flex-row items-center w-full gap-2">
      <Progress :model-value="downloadProgress" class="w-full" />
    </div>
    <div class="w-full flex-1 overflow-y-auto border border-gray-200 rounded p-2 bg-slate-50" ref="logArea">
      <p class="text-sm font-mono" v-for="log in downloadLog" :key="log">{{ log }}</p>
    </div>
    <div class="absolute bottom-4 right-4">
      <Button :disabled="downloading" @click="downloadVideo"><span class="material-icons">cloud_download</span>{{
        downloading ? 'ダウンロード中...' : 'ダウンロード' }}</Button>
    </div>
  </main>
</template>