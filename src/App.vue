<script setup lang="ts">
// import
import { ref, onMounted, nextTick, watch, computed } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { homeDir } from '@tauri-apps/api/path';
import { Command } from '@tauri-apps/plugin-shell';
import { platform } from '@tauri-apps/plugin-os';

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

// いろいろ準備するやつ
const savePath = ref<string>('');
const logArea = ref<HTMLElement | null>(null);
const videoUrl = ref('');
const downloadProgress = ref<number | null>(0);
const downloadTitle = ref('')
const downloading = ref(false)
const downloadLog = ref<string[]>([])
const downloadErrors = ref<string[]>([])
const currentOS = platform()
const playlistMode = ref(false)
const namedIndex = ref(false)
const exts = ref<string[]>(['mp4', 'mkv', 'mp3', 'flac', 'wav'])
const selectedExt = ref<string>('mp4')
const videoQualitys = ref<QualityItem[]>([{ label: '自動', value: 'auto' }, { label: '4K', value: '2160' }, { label: '2K', value: '1440' }, { label: '1080p', value: '1080' }, { label: '720p', value: '720' }])
const mp3Qualitys = ref<QualityItem[]>([{ label: '自動', value: 'auto' }, { label: '320Kbps', value: '320' }, { label: '256Kbps', value: '256' }, { label: '192Kbps', value: '192' }, { label: '128Kbps', value: '128' }])
const selectedQuality = ref<string>('auto')

const qualityOptions = computed<QualityItem[]>(() => {
  if (['mp3'].includes(selectedExt.value)) {
    return mp3Qualitys.value
  } else if (['mp4', 'mkv'].includes(selectedExt.value)) {
    return videoQualitys.value
  } else {
    return []
  }
})

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
    savePath.value = await homeDir();
    console.log(savePath.value)
    downloadLog.value.push('[🚀] yt-dlpの更新を確認しています...')
    const updateYTDLP = Command.sidecar('binaries/yt-dlp', ['-U'])
    await updateYTDLP.spawn()
    updateYTDLP.on('close', (data) => {
      console.log('yt-dlp update process closed with code:', data.code)
    })
    updateYTDLP.stdout.on('data', (line: string) => {
      console.log('[UPDATE]', line.trim())
      downloadLog.value.push('[🚀] ' + line.trim())
    })
  } catch (err) {
    console.error('Error:', err)
  }
})

// 保存先選択処理
const selectSaveDir = async () => {
  if (!savePath.value) {
    savePath.value = await homeDir();
  }

  const beforeDir = savePath.value

  const dir = await open({
    multiple: false,
    directory: true
  })

  if (typeof dir === 'string') {
    savePath.value = dir
  } else {
    savePath.value = beforeDir
  }
}

// ダウンロード処理
const downloadVideo = async () => {
  if (!videoUrl.value) {
    downloadLog.value.push('[❌️] URLが入力されていません')
    return
  }

  downloadLog.value.push('[⬇️] ダウンロードを開始します...')
  downloadProgress.value = 0
  downloading.value = true
  downloadTitle.value = ''
  downloadErrors.value = []
  const progress_template = '[DOWNLOADING]::%(progress._percent)s::%(info.title)s'
  const encoding = (await currentOS) === 'windows' ? 'shift_jis' : 'utf-8'
  let env: Record<string, string> = {}
  if (await currentOS === 'macos') {
    env['PATH'] = '/opt/homebrew/bin:/usr/local/bin:'
  }
  const ytdlopts = ['--newline', '--no-color', '--progress-template', progress_template]
  if (['mp4', 'mkv'].includes(selectedExt.value)) {
    if (selectedQuality.value === 'auto') {
      ytdlopts.push('-f', 'bestvideo+bestaudio[ext=m4a]/best')
    } else if (selectedQuality.value) {
      ytdlopts.push('-f', `bestvideo[height<=?${selectedQuality.value}]+bestaudio[ext=m4a]/best[height<=?${selectedQuality.value}]`)
    } else {
      ytdlopts.push('-f', 'bestvideo+bestaudio[ext=m4a]/best')
    }
    ytdlopts.push('--merge-output-format', selectedExt.value)
  } else if (selectedExt.value === 'mp3') {
    ytdlopts.push('-f', 'bestaudio/best','-x', '--audio-format', 'mp3')
    if (selectedQuality.value && selectedQuality.value !== 'auto') {
      ytdlopts.push('--audio-quality', selectedQuality.value)
    } else {
      ytdlopts.push('--audio-quality', '0')
    }
  } else if (selectedExt.value === 'flac') {
    ytdlopts.push('-f', 'bestaudio/best','-x', '--audio-format', 'flac')
  } else if (selectedExt.value === 'wav') {
    ytdlopts.push('-f', 'bestaudio/best','-x', '--audio-format', 'wav')
  }
  if (playlistMode.value) {
    if (namedIndex.value) {
      ytdlopts.push('-o', savePath.value + '/%(playlist_title)s/%(playlist_index)s - %(title)s.%(ext)s')
    } else {
      ytdlopts.push('-o', savePath.value + '/%(playlist_title)s/%(title)s.%(ext)s')
    }
  } else {
    ytdlopts.push('-o', savePath.value + '/%(title)s.%(ext)s')
  }
  ytdlopts.push(videoUrl.value)
  console.log('yt-dlp options:', ytdlopts)
  const cmd = Command.sidecar('binaries/yt-dlp', ytdlopts, { encoding: encoding, env: env })
  cmd.stdout.on('data', (line: string) => {
    if (line.startsWith('[DOWNLOADING]')) {
      const parts = line.trim().split('::')
      if (parts.length >= 3) {
        const percentStr = parts[1].trim()
        downloadProgress.value = parseFloat(percentStr)
        if (!downloadTitle.value || downloadTitle.value !== parts[2].trim()) {
          downloadTitle.value = parts[2].trim()
        }
      }
    } else {
      downloadLog.value.push(line.trim())
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
  <main class="p-6 h-screen min-h-screen relative">
    <h1 class="text-2xl font-bold">Syt</h1>
    <div class="flex flex-col items-start gap-2 mt-4 w-full">
      <div class="flex flex-row items-center w-full gap-2">
        <Input v-model="videoUrl" type="text" placeholder="URLを入力..." />
      </div>
      <div class="flex flex-row items-center w-full gap-2">
        <Input v-model="savePath" type="text" placeholder="保存先" readonly />
        <Button @click="selectSaveDir"><span class="material-icons">folder</span>保存先を選択</Button>
      </div>
      <h2 class="text-lg font-bold">オプション</h2>
      <div class="flex flex-col sm:flex-row items-center w-full gap-2">
        <div class="flex flex-row items-center w-full">
          <Label for="extSelect" class="whitespace-nowrap mr-2">拡張子</Label>
          <Select v-model="selectedExt" class="w-32" id="extSelect">
            <SelectTrigger class="w-full">
              <SelectValue placeholder="拡張子を選択" />
            </SelectTrigger>
            <SelectContent>
              <SelectLabel>拡張子を選択</SelectLabel>
              <SelectItem v-for="ext in exts" :value="ext" :key="ext">{{ ext }}</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div class="flex flex-row items-center w-full">
          <Label for="qualitySelect" class="whitespace-nowrap mr-2">品質</Label>
          <Select v-model="selectedQuality" class="w-32" id="qualitySelect">
            <SelectTrigger class="w-full">
              <SelectValue placeholder="品質を選択" />
            </SelectTrigger>
            <SelectContent>
              <SelectLabel>{{ selectedExt === 'mp3' ? '音質を選択' : '画質を選択' }}</SelectLabel>
              <SelectItem v-for="quality in qualityOptions" :value="quality.value" :key="quality.value">{{ quality.label }}</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
      <div class="flex flex-col sm:flex-row items-start sm:items-center w-full gap-2">
        <div class="flex flex-row gap-2">
          <Switch v-model="playlistMode" id="playlistSwitch" />
          <Label for="playlistSwitch">プレイリストモード</Label>
        </div>
        <div class="flex flex-row gap-2">
          <Switch :disabled="!playlistMode" v-model="namedIndex" id="namedIndexSwitch" />
          <Label for="namedIndexSwitch">インデックスをファイル名に追加する</Label>
        </div>

      </div>
      <h2 class="text-lg font-bold">ログ</h2>
      <div class="flex flex-row items-center w-full gap-2">
        <Progress :model-value="downloadProgress" class="w-full" />
      </div>
      <div class="w-full h-40 overflow-y-auto border border-gray-200 rounded-sm p-2" ref="logArea">
        <p class="text-sm font-mono" v-for="log in downloadLog" :key="log">{{ log }}</p>
      </div>
    </div>
    <div class="absolute bottom-4 right-4">
      <Button :disabled="downloading" @click="downloadVideo"><span
          class="material-icons">cloud_download</span>ダウンロード開始</Button>
    </div>
  </main>
</template>