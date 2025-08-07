<template>
  <div class="pico update-container">
    <dialog open>
      <div class="container">
        <h2>A new version is available!</h2>
        <sb-icon icon="close" size="32px" @click="$emit('close')" />
        <div role="group">
          <button v-if="readyToRestart" @click="relaunch()">
            <sb-icon icon="restart_alt" color="inherit" />
            Relaunch the app
          </button>
          <button v-else @click="install()">
            <sb-icon icon="install_desktop" color="inherit" />
            Install Now
          </button>
          <button class="secondary"
            @click="openUrl(`https://github.com/fatfish-lab/steamboard-private/releases/${updater.update.version}`)">
            <sb-icon icon="info" color="inherit" />
            See what's new
          </button>
        </div>
        <progress v-if="contentLength > 0" :value="progression" max="100" />
        <span v-if="contentLength > 0">{{ progression }}%</span>
        <span class="current-version">You are currently using <strong>v{{ updater.version }}</strong>.</span>
      </div>
    </dialog>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener';
import { useUpdaterStore } from '@/stores/updater.ts';
import { relaunch } from '@tauri-apps/plugin-process';


const alert = inject('alert')
const updater = useUpdaterStore();
const contentLength = ref(0);
const downloaded = ref(0);
const readyToRestart = ref(false);

const progression = computed(() => {
  if (contentLength.value === 0) return 0;
  return Math.round((downloaded.value / contentLength.value) * 100);
});

async function install() {
  downloaded.value = 0;
  contentLength.value = 0;
  readyToRestart.value = false;

  const update = await updater.check();
  if (update) {
    await update.downloadAndInstall(event => {
      switch (event.event) {
        case 'Started':
          contentLength.value = event.data.contentLength;
          break;
        case 'Progress':
          downloaded.value += event.data.chunkLength;
          break;
        case 'Finished':

          readyToRestart.value = true;
          break;
      }
    }).catch(e => {
      readyToRestart.value = false;
      downloaded.value = 0;
      contentLength.value = 0;
      // console.error('Update failed:', e);
      alert(e);
    });


  }
}
</script>

<style lang="scss" scoped>
.update-container {
  dialog {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;

    div.container {
      display: grid;
      grid-template-columns: 1fr min-content;
      grid-template-rows: min-content min-content;
      align-items: center;
      justify-items: center;
      gap: 32px 8px;
      background-color: var(--pico-background-color);
      padding: 32px;
      border-radius: 8px;
      max-width: 50vw;
      text-align: center;

      h2 {
        margin: 0;
      }

      [role="group"],
      .current-version {
        grid-column: 1 / -1;
      }

      strong {
        color: var(--pico-primary);
      }

      progress {
        margin: 0;
      }
    }
  }
}

</style>