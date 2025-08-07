<template>
  <main class="container-fluid">
    <RouterView class="view-container" />
    <footer>
      <section>
        Powered by <span @click="openUrl('https://fatfi.sh')">Fatfish Lab</span> & <span
          @click="openUrl('https://bubblebirdstudio.com')">Bubblebird Studio</span>
        -
        <a class="support invisible-a" @click="openUrl('https://steamboard.app/#support')">Looking for help ? Get
          support !</a>
      </section>
      <!-- <section class="update">
        <span v-if="isUpdateAvailable" @click="showUpdate = true">
          <sb-icon icon="release_alert" color="amber-300" fill />
          New version available
        </span>
      </section> -->
      <section>
        <sb-icon icon="verified_user" color="green-300" fill />
        <span @click="openUrl('https://steamboard.app/#privacy')">Privacy Statement</span>
      </section>
    </footer>
    <Waves class="waves" />
  </main>
  <div class="alerts">
    <sb-alert
      v-for="(alert, index) in alerts"
      :key="index"
      :title="alert.title"
      :message="alert.message"
      @close="alerts.splice(index, 1)" />
  </div>
  <!-- <Update v-if="showUpdate" @close="showUpdate = false" /> -->
</template>


<script setup lang="ts">
import { ref, onMounted, provide, computed } from "vue";
import { useRouter, useRoute } from 'vue-router';
import { useSettingsStore } from '@/stores/settings.ts';
import { listen } from '@tauri-apps/api/event';
import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';
import Waves from '@/components/Waves.vue';

import { useUpdaterStore } from '@/stores/updater.ts';
// import Update from '@/components/Update.vue';

const router = useRouter();
const settings = useSettingsStore();
// const showUpdate = ref(false);
const updater = useUpdaterStore();
// const isUpdateAvailable = computed(() => updater.isUpdateAvailable);


const alerts = ref<{ title: string; message: string }[]>([]);

listen("settings-updated", (event) => {
  init()
})

onMounted(async () => {
  updater.version = await getVersion();
  // updater.check();
  init();
});

async function init() {
  await settings.load();

  if (settings.steam_api_key && settings.loaded) {
    router.push({ name: 'overview' });
  } else if (settings.steam_api_key === '') {
    router.push({ name: 'decrypting' });
  } else {
    router.push({ name: 'welcome' });
  }
}

function alert(error: Error) {
  const title = error.name || 'Error';
  const message = error.message || (typeof error === 'string' ? error : null )|| 'An unexpected error occurred';
  alerts.value.push({ title, message });
}
provide('alert', alert);


</script>

<style lang="scss">
@use "@picocss/pico/scss/pico" with ($parent-selector: ".pico",
  $theme-color: "amber"
);
@use "@picocss/pico/scss/pico.colors" as picoColors;

@use "./reset.scss";
@use "./style.scss";
</style>

<style lang="scss" scoped>
main {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: 1fr min-content;
  gap: 8px;
  align-items: center;
  justify-items: center;
  height: 100%;
  padding: 16px;
  min-height: 0;
  position: relative;

  .waves {
    position: absolute;
    bottom: 0;
    right: 0;
    width: 60%;
    z-index: -1;
    transform: scaleX(-100%);
  }

  .view-container {
    height: 100%;
    width: 100%;
    padding: 48px;
    min-height: 0;
    overflow: auto;
  }

  footer {
    display: inline-grid;
    grid-template-columns: 1fr max-content;
    gap: 8px;
    align-items: center;
    width: 100%;
    padding: 0 16px;
    // text-align: center;
    font-size: 0.7rem;
    color: color-mix(in oklab, var(--pico-muted-color), var(--pico-muted-border-color) 50%);

    >section>span {
      cursor: pointer;

      &:hover {
        color: var(--pico-primary);
      }
    }

    section.update {
      >span {
        display: inline-flex;
        align-items: center;
        gap: 4px;
        color: var(--pico-primary);
        cursor: pointer;
        position: relative;
        background-color: color-mix(in oklab, var(--pico-background-color), var(--pico-muted-border-color) 30%);
        padding: 8px;
        border-radius: 4px;

        &:after {
            content: '';
            position: absolute;
            width: 8px;
            height: 8px;
            top: -3px;
            right: -3px;
            background-color: var(--pico-color-red-400);
            border-radius: 8px;
          }

        &:hover {
          color: var(--pico-primary-hover);
          background-color: var(--pico-muted-border-color);
        }
      }
    }

    section {
      a.support {
        color: var(--pico-primary);
        cursor: pointer;
      }
    }

    section {
      display: inline-flex;
      align-items: center;
      gap: 4px;
    }

  }
}

.alerts {
  display: flex;
  flex-direction: column;
  gap: 8px;
  position: absolute;
  bottom: 16px;
  right: 16px;
  z-index: 1000;
}
</style>