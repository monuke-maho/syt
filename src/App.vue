<script setup lang="ts">
  import { ref, onMounted } from 'vue';
  import { open } from '@tauri-apps/plugin-dialog';
  import { homeDir } from '@tauri-apps/api/path';
  import { Command } from '@tauri-apps/plugin-shell';
  const savePath = ref<string | null>(null);
  const videoUrl = ref('');
  const downloadProgress = ref(0)
  const downloadTitle = ref('')
  const downloading = ref(false)

  onMounted(async () => {
    try {
      savePath.value = await homeDir();
      console.log(savePath.value)
    } catch (err) {
      console.error('Error:',err)
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
    downloading.value = true
    downloadTitle.value = ''
    const progress_template = '[DOWNLOADING]::%(progress._percent)s::%(info.title)s'
    const cmd = Command.create('yt-dlp',['--newline',videoUrl.value,'-o',savePath.value+'/%(title)s.%(ext)s','-f','bestvideo+bestaudio/best','--merge-output-format','mp4','--progress-template', progress_template])
    cmd.on('close', data => console.log(data))
    cmd.on('error', err => console.error(err))
    cmd.stdout.on('data', line => {
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
        console.log(line.trim())
      }
    })
    await cmd.spawn()
    cmd.on('close', () => {
      downloading.value = false
      downloadTitle.value = ''
      downloadProgress.value = 0
    })
  }
</script>

<template>
  <main class="p-6 h-screen min-h-screen">
    <h1 class="text-2xl font-bold">syt</h1>
    <div class="container flex flex-col items-start gap-4 mt-2 w-full">
      <div class="flex flex-row items-center w-full gap-2">
        <input type="text" name="url" id="url" placeholder="URLを入力..." v-model="videoUrl" class="p-2 border border-gray-300 rounded-sm w-full">
        <button :class="downloading ? 'p-2 rounded-sm bg-gray-500 cursor-not-allowed' : 'p-2 rounded-sm bg-blue-500 text-white cursor-pointer'" @click="downloadVideo">{{ downloading ? `Downloading...` : `Download` }}</button>
      </div>
      <div class="flex flex-row items-center w-full gap-2">
        <input type="text" name="url" id="url" readonly placeholder="保存先" v-model="savePath" class="p-2 border border-gray-300 rounded-sm w-full">
        <button class="p-2 rounded-sm bg-blue-500 text-white cursor-pointer" @click="selectSaveDir">選択</button>
      </div>
    </div>
  </main>
</template>