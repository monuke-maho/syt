<script setup lang="ts">
// import
import { ref, reactive, onMounted, nextTick, watch, computed } from 'vue';
import { open,confirm } from '@tauri-apps/plugin-dialog';
import { homeDir,join } from '@tauri-apps/api/path';
import { Command } from '@tauri-apps/plugin-shell';
import { platform } from '@tauri-apps/plugin-os';
import { LazyStore } from '@tauri-apps/plugin-store';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { getVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

import { Button } from './components/ui/button';
import { Input } from './components/ui/input';
import { Progress } from './components/ui/progress';
import { Switch } from './components/ui/switch';
import { Label } from './components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './components/ui/tabs';

interface BrowserProfile {
  browser_name: string;
  profile_name: string;
  root_directory: string;
}

const allProfiles = ref<BrowserProfile[]>([]);
const errMsg = ref('');
const fetchProfiles = async () => {
  try {
    const data = await invoke<BrowserProfile[]>('get_all_profiles');
    allProfiles.value = data;
  } catch (err) {
    console.error(err);
    errMsg.value = 'プロファイルの取得に失敗';
  }
}

const browserList = computed<string[]>(() => {
  const names = allProfiles.value.map(p => p.browser_name).filter(name => name !== '');
  return [...new Set(names)]
})

const filteredProfiles = computed(() => {
  return allProfiles.value.filter(p => p.browser_name === settings.selectedBrowser)
})

interface QualityItem {
  label: string;
  value: string;
}

interface AppSettings {
  savePath: string;
  selectedExt: string;
  selectedQuality: string;
  selectedBrowser: string;
  selectedProfile: string;
  customPath: string;
  playlistMode: boolean;
  namedIndex: boolean;
  embedThumbnails: boolean;
  cropdThumbnails: boolean;
  embedMetadata: boolean;
  albumMode: boolean;
  autoUpdate: boolean;
}

const appVersion = ref('')
const SETTINGS_KEY = 'app-config';
const store = new LazyStore('settings.json')
const currentOS = platform()
const denoPath = ref('')

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
  selectedBrowser: '',
  selectedProfile: '',
  customPath: '',
  playlistMode: false,
  namedIndex: false,
  embedThumbnails: false,
  cropdThumbnails: false,
  embedMetadata: false,
  albumMode: false,
  autoUpdate: true
})

const videoUrl = ref('');
const downloading = ref(false);
const downloadProgress = ref<number | null>(0);
const status = ref('ステータス');
const downloadLog = ref<string[]>([]);
const downloadErrors = ref<string[]>([]);
const logArea = ref<HTMLElement | null>(null);
const activeTab = ref('options');

const qualityOptions = computed(() => {
  if (settings.selectedExt === 'mp3') {
    return AUDIO_QUALITIES.map(q => ({ ...q, stableKey: `audio-${q.value}` }));
  } else if (['mp4', 'mkv'].includes(settings.selectedExt)) {
    return VIDEO_QUALITYS.map(q => ({ ...q, stableKey: `video-${q.value}` }));
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

watch(() => settings.selectedBrowser, (newVal) => {
  if (newVal === 'none' || newVal === '') {
    settings.selectedProfile = '';
  }
})

// 起動時の処理
onMounted(async () => {
  try {
    fetchProfiles();
    await store.init();
    appVersion.value = await getVersion()

    const saved = await store.get<AppSettings>(SETTINGS_KEY);
    const defaultPath = await homeDir();
    denoPath.value = await join(defaultPath,'.deno','bin')

    if (saved && typeof saved === 'object') {
      if ('savePath' in saved) {
        Object.assign(settings, saved);
      } else {
        settings.savePath = defaultPath;
      }
    } else {
      settings.savePath = defaultPath;
    }

    // すべての設定が反映された後に初期化完了とする
    await nextTick();
    isInit.value = true;
    console.log('App Initialized');

    if (settings.autoUpdate) {
      addLog('[🚀] yt-dlpの更新を確認中...')
      const updateCmd = Command.sidecar('binaries/yt-dlp', ['-U']);
      updateCmd.stdout.on('data', (line) => addLog(`[UPDATE] ${line.trim()}`))
      await updateCmd.spawn()
    }
    
    addLog('[🔄] アプリの更新を確認中...')
    try {
      const update = await check();
      if (update) {
        addLog(`[👀] 更新が見つかりました! バージョン: ${update.version}`)
        const confirmUpdate = await confirm(`更新が見つかりました(v${update.version})。更新しますか?`, { title: "確認", kind: "info" })
        if (confirmUpdate) {
            addLog('[⬇️] 更新しています...')
            await update.downloadAndInstall()
            await relaunch()
        }
      } else {
        addLog('[✅] 最新バージョンです')
      }
    } catch (err) {
      addLog(`[❌] 更新確認エラー: ${err}`)
      console.error('Update check error:', err)
    }
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
  let permissionGranted = await isPermissionGranted();
  if (!permissionGranted) {
    const notifyPermission = await requestPermission();
    permissionGranted = notifyPermission === 'granted';
    console.log('Notification permission:', notifyPermission);
  }

  downloading.value = true;
  downloadProgress.value = 0;
  status.value = 'ダウンロードを開始します...';
  downloadErrors.value = [];
  activeTab.value = 'logs'; // Switch to logs tab
  addLog('[⬇️] ダウンロードを開始します...')

  const isAudio = ['mp3', 'flac', 'wav'].includes(settings.selectedExt);
  const encoding = currentOS === 'windows' ? 'shift_jis' : 'utf-8';
  const env: Record<string, string> = currentOS === 'macos' || currentOS === 'linux' ? { PATH: `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/usr/sbin:/sbin:${denoPath.value}` } : {}
  const ytdlopts = ['--newline', '--no-color', '--progress-template', '[DOWNLOADING]::%(progress._percent)s::%(info.title)s','--remote-components','ejs:npm']

  if (settings.albumMode && isAudio) {
    ytdlopts.push('-f', 'bestaudio/best', '-x', '--audio-format', settings.selectedExt);
    if (settings.selectedExt === 'mp3') {
      const q = settings.selectedQuality === 'auto' ? '0' : settings.selectedQuality;
      ytdlopts.push('--audio-quality', q)
    }
    let outputTemplate = settings.savePath;
    outputTemplate += '/%(album)s/%(playlist_index)02d - %(title)s.%(ext)s';
    ytdlopts.push('-o', outputTemplate)
    if (settings.selectedExt !== 'wav') {
      ytdlopts.push('--embed-thumbnail', "--convert-thumbnails", "jpg", "--ppa", "ThumbnailsConvertor:-qmin 1 -q:v 1 -vf crop=\"'if(gt(ih,iw),iw,ih)':'if(gt(iw,ih),ih,iw)'\"")
    }
    ytdlopts.push('--add-metadata', '--parse-metadata', '%(playlist_index)s/%(n_entries)s:%(track_number)s', '--parse-metadata', '%(release_date).4s:%(meta_date)s', '--parse-metadata', '%(creators.0)s:%(meta_album_artist)s')
  } else {
    // 処理
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
    if (settings.embedMetadata) {
      ytdlopts.push('--add-metadata')
    }
    ytdlopts.push('-o', outputTemplate)
  }

  if (settings.selectedBrowser && settings.selectedBrowser !== 'none' && settings.selectedProfile) {
    if (settings.customPath && settings.selectedProfile === 'custom') {
      ytdlopts.push("--cookies-from-browser", `firefox:${settings.customPath}`)
    } else {
      ytdlopts.push("--cookies-from-browser", `firefox:${settings.selectedProfile}`)
    }
  }
  ytdlopts.push(videoUrl.value)

  console.log('yt-dlp options:', ytdlopts)
  const cmd = Command.sidecar('binaries/yt-dlp', ytdlopts, { encoding: encoding, env: env })
  cmd.stdout.on('data', (line: string) => {
    const trimmed = line.trim()
    if (trimmed.startsWith('[DOWNLOADING]')) {
      const [, percent, title] = trimmed.split('::');
      downloadProgress.value = parseFloat(percent)
      if (title) status.value = title + ' をダウンロード中...'
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
    status.value = 'ダウンロードが完了しました！'
    if (data.code === 0) {
      downloadProgress.value = 100
      downloadLog.value.push('[✅] ダウンロードが完了しました！')
      if (permissionGranted) {
        sendNotification({ title: 'ダウンロード完了', body: 'ダウンロードを完了しました。' })
      }
    } else if (data.code === 1) {
      downloadProgress.value = 0
      status.value = 'ダウンロード中にエラーが発生しました'
      downloadLog.value.push('[❌] ダウンロード中にエラーが発生しました。')
      downloadErrors.value.forEach((err) => {
        downloadLog.value.push('[ERROR] ' + err)
      })
      if (permissionGranted) {
        sendNotification({ title: 'エラー発生', body: 'ダウンロード中にエラーが発生しました。' })
      }
    }
  })
}
</script>

<template>
  <main class="p-6 h-screen min-h-screen relative flex flex-col gap-2 **:select-none">
    <h1 class="text-2xl font-bold">Syt</h1>

    <!-- URL Input -->
    <div class="flex flex-row items-center w-full gap-2">
      <Input v-model="videoUrl" type="text" placeholder="URLを入力..." />
      <Button :disabled="downloading" @click="downloadVideo"><span class="material-icons">{{ downloading ?
        'downloading'
        : 'download' }}</span>{{ downloading ? 'ダウンロード中' : 'ダウンロード' }}</Button>
    </div>

    <!-- Progress Bar (Always Visible) -->
    <div v-if="downloading || downloadProgress !== null" class="flex flex-col gap-1">
      <div class="flex flex-row items-center w-full gap-2">
        <Progress :model-value="downloadProgress" class="w-full" />
      </div>
      <p class="text-sm text-muted-foreground truncate">{{ status }}</p>
    </div>

    <!-- Save Path -->
    <div class="flex flex-row items-center w-full gap-2">
      <Input v-model="settings.savePath" type="text" placeholder="保存先" readonly />
      <Button @click="selectSaveDir"><span class="material-icons">folder</span>保存先を選択</Button>
    </div>

    <!-- Tabs for Options and Logs -->
    <Tabs v-model="activeTab" class="flex-1 flex flex-col overflow-hidden">
      <TabsList class="grid w-full grid-cols-3">
        <TabsTrigger value="options">オプション</TabsTrigger>
        <TabsTrigger value="logs">ログ</TabsTrigger>
        <TabsTrigger value="settings">設定</TabsTrigger>
      </TabsList>

      <TabsContent value="options" class="flex-1 overflow-y-auto mt-4 space-y-6 pr-2">

        <section class="space-y-3">
          <div class="flex items-center gap-2 mb-1">
            <span class="material-icons text-sm">settings</span>
            <h3 class="text-sm font-semibold">基本設定</h3>
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 bg-secondary/30 p-4 rounded-lg">
            <div class="space-y-2">
              <Label for="extSelect" class="text-xs font-medium">拡張子</Label>
              <Select v-model="settings.selectedExt" id="extSelect">
                <SelectTrigger class="w-full bg-background">
                  <SelectValue placeholder="選択" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="ext in EXTS" :value="ext" :key="ext">{{ ext }}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div class="space-y-2">
              <Label for="qualitySelect" class="text-xs font-medium">品質</Label>
              <Select v-model="settings.selectedQuality" id="qualitySelect">
                <SelectTrigger class="w-full bg-background">
                  <SelectValue placeholder="選択" />
                </SelectTrigger>
                <SelectContent v-if="isInit">
                  <SelectItem v-for="quality in qualityOptions" :value="quality.value" :key="quality.stableKey">{{
                    quality.label }}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div class="space-y-2">
              <Label>ブラウザー</Label>
              <Select v-model="settings.selectedBrowser">
                <SelectTrigger class="w-full bg-background">
                  <SelectValue placeholder="選択" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">使用しない</SelectItem>
                  <SelectItem v-for="name in browserList" :value="name" :key="name">{{ name }}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div class="space-y-2">
              <Label>プロファイル</Label>
              <Select v-model="settings.selectedProfile"
                :disabled="settings.selectedBrowser === 'none' || settings.selectedBrowser === ''">
                <SelectTrigger class="w-full bg-background">
                  <SelectValue placeholder="選択" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="p in filteredProfiles" :value="p.root_directory" :key="p.root_directory">{{
                    p.profile_name }}</SelectItem>
                  <SelectItem value="custom">パスを手動入力</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div class="space-y-2 col-span-2" v-if="settings.selectedProfile === 'custom'">
              <Label for="customPath">プロファイルパス</Label>
              <Input v-model="settings.customPath" type="text" id="customPath" />
            </div>
          </div>
        </section>

        <section class="space-y-3">
          <div class="flex items-center gap-2 mb-1">
            <span class="material-icons text-sm">playlist_play</span>
            <h3 class="text-sm font-semibold">プレイリスト・アルバム</h3>
          </div>
          <div class="grid grid-cols-1 gap-1 border rounded-lg divide-y">
            <div class="flex items-center justify-between p-3">
              <div class="space-y-0.5">
                <Label for="playlistSwitch" class="text-sm font-medium">プレイリストモード</Label>
                <p class="text-xs text-muted-foreground">複数の動画をまとめて取得します</p>
              </div>
              <Switch v-model="settings.playlistMode" id="playlistSwitch" />
            </div>
            <div class="flex items-center justify-between p-3" :class="{ 'opacity-50': !settings.playlistMode }">
              <div class="space-y-0.5">
                <Label for="namedIndexSwitch" class="text-sm font-medium">インデックスを付与</Label>
                <p class="text-xs text-muted-foreground">ファイル名の先頭に番号を追加</p>
              </div>
              <Switch :disabled="!settings.playlistMode" v-model="settings.namedIndex" id="namedIndexSwitch" />
            </div>
            <div class="flex items-center justify-between p-3"
              :class="{ 'opacity-50': !['mp3', 'flac', 'wav'].includes(settings.selectedExt) }">
              <div class="space-y-0.5">
                <Label for="albumSwitch" class="text-sm font-medium">アルバムモード</Label>
                <p class="text-xs text-muted-foreground">オーディオ形式時のみ有効</p>
              </div>
              <Switch :disabled="!['mp3', 'flac', 'wav'].includes(settings.selectedExt)" v-model="settings.albumMode"
                id="albumSwitch" />
            </div>
          </div>
        </section>

        <section class="space-y-3">
          <div class="flex items-center gap-2 mb-1">
            <span class="material-icons text-sm">audio_file</span>
            <h3 class="text-sm font-semibold">メタデータ・サムネイル</h3>
          </div>
          <div class="grid grid-cols-1 gap-1 border rounded-lg divide-y">
            <div class="flex items-center justify-between p-3">
              <Label for="embedMetadataSwitch" class="text-sm font-medium">メタデータを埋め込む</Label>
              <Switch v-model="settings.embedMetadata" id="embedMetadataSwitch" />
            </div>
            <div class="flex items-center justify-between p-3">
              <Label for="embedThumbnailsSwitch" class="text-sm font-medium">サムネイルを埋め込む</Label>
              <Switch v-model="settings.embedThumbnails" id="embedThumbnailsSwitch" />
            </div>
            <div class="flex items-center justify-between p-3" :class="{ 'opacity-50': !settings.embedThumbnails }">
              <Label for="cropThumbnailsSwitch" class="text-sm font-medium text-muted-foreground ml-4">└
                1:1にクロップ</Label>
              <Switch :disabled="!settings.embedThumbnails" v-model="settings.cropdThumbnails"
                id="cropThumbnailsSwitch" />
            </div>
          </div>
        </section>

      </TabsContent>

      <!-- Logs Tab -->
      <TabsContent value="logs" class="flex-1 overflow-hidden mt-4">
        <div class="w-full h-full overflow-y-auto border border-gray-200 rounded p-2 bg-slate-50" ref="logArea">
          <p class="text-sm font-mono select-text!" v-for="(log, idx) in downloadLog" :key="idx">{{ log }}</p>
        </div>
      </TabsContent>
      <TabsContent value="settings" class="flex-1 overflow-y-auto mt-4 space-y-6 pr-2">
        <section class="space-y-3">
          <div class="flex items-center gap-2 mb-1">
            <span class="material-icons text-sm">settings</span>
            <h3 class="font-bold text-sm">基本設定</h3>
          </div>
          <div class="grid grid-cols-1 gap-1 border rounded-lg divide-y">
            <div class="flex items-center justify-between p-3">
              <Label for="autoUpdateSwitch" class="text-sm font-medium">起動時に更新を確認する</Label>
              <Switch v-model="settings.autoUpdate" id="autoUpdateSwitch" />
            </div>
          </div>
        </section>
        <section class="space-y-3">
          <div class="flex items-center gap-2 mb-1">
            <h2 class="text-lg font-bold">About</h2>
          </div>
          <div class="flex flex-col items-start gap-1">
            <p>バージョン: {{ appVersion }}</p>
            <p>開発者: monuke-maho</p>
            <a href="https://github.com/monuke-maho/syt" target="_blank" class="underline">GitHub</a>
          </div>
        </section>
      </TabsContent>
    </Tabs>
  </main>
</template>