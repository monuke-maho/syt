<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { homeDir } from '@tauri-apps/api/path';
import { Command } from '@tauri-apps/plugin-shell';
const savePath = ref<string | null>(null);
const logArea = ref<HTMLElement | null>(null);
const videoUrl = ref('');
const downloadProgress = ref(0)
const downloadTitle = ref('')
const downloading = ref(false)
const downloadLog = ref<string[]>([])

watch(downloadLog, async () => {
  await nextTick();
  if (logArea.value) {
    logArea.value.scrollTop = logArea.value.scrollHeight;
  }
}, { deep: true })

onMounted(async () => {
  try {
    savePath.value = await homeDir();
    console.log(savePath.value)
  } catch (err) {
    console.error('Error:', err)
  }
})

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

const downloadVideo = async () => {
  downloadProgress.value = 0
  downloading.value = true
  downloadTitle.value = ''
  const progress_template = '[DOWNLOADING]::%(progress._percent)s::%(info.title)s'
  const cmd = Command.create('yt-dlp', ['--no-color', '--newline', videoUrl.value, '-o', savePath.value + '/%(title)s.%(ext)s', '-f', 'bestvideo[ext=mp4]+bestaudio[ext=m4a]/best', '--merge-output-format', 'mp4', '--progress-template', progress_template], { encoding: 'shift_jis' })
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
    }
  })
  cmd.stderr.on('data', (line: string) => {
    console.error(line.trim())
  })
  
  await cmd.spawn()
  cmd.on('close', (data) => {
    downloading.value = false
    downloadTitle.value = ''
    if (data.code === 0) {
      downloadProgress.value = 100
    } else if (data.code === 1) {
      downloadProgress.value = 0
    }
  })
}
</script>

<template>
  <main class="p-6 h-screen min-h-screen relative">
    <h1 class="text-2xl font-bold">Syt</h1>
    <div class="flex flex-col items-start gap-2 mt-4 w-full">
      <div class="flex flex-row items-center w-full gap-2">
        <input type="text" name="url" id="url" placeholder="URLを入力..." v-model="videoUrl"
          class="p-2 border border-gray-300 rounded-sm w-full">
      </div>
      <div class="flex flex-row items-center w-full gap-2">
        <input type="text" name="url" id="url" readonly placeholder="保存先" v-model="savePath"
          class="p-2 border border-gray-300 rounded-sm w-full">
        <a class="p-2 whitespace-nowrap font-bold rounded-sm bg-blue-500 text-white cursor-pointer flex flex-row items-center gap-2"
          @click="selectSaveDir"><span class="material-icons">folder</span> 保存先を選択</a>
      </div>
      <div class="flex flex-row items-center w-full gap-2">
        <div class="w-full h-2 bg-gray-200 rounded-full">
          <div class="h-full bg-blue-500 rounded-full transition-all duration-75"
            :style="{ width: `${downloadProgress}%` }"></div>
        </div>
      </div>
      <div class="w-full h-40 overflow-y-auto border border-gray-200 rounded-sm p-2" ref="logArea">
        <p class="text-sm font-mono" v-for="log in downloadLog" :key="log">{{ log }}</p>
      </div>
    </div>
    <div class="absolute bottom-4 right-4">
      <button class="flex flex-row items-center gap-2 w-fit transition-all duration-300"
        :class="downloading ? 'p-2 rounded-sm bg-gray-500 cursor-not-allowed' : 'p-2 rounded-sm bg-blue-500 text-white cursor-pointer'"
        @click="downloadVideo"><span class="material-icons">{{ downloading ? 'hourglass_empty' : 'download' }}</span> {{
          downloading ? `Downloading...` : `Download` }}</button>
    </div>
  </main>
</template>