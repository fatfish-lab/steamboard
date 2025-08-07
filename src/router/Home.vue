<template>
  <div class="home-container">
    <header>
      <Toolbar />
      <nav>
        <router-link role="button" v-for="page in pages" :key="page.name" :to="{ name: page.name }"
          class="invisible-a"
          :class="{ 'active': activePage === page.name }">
          <sb-icon :icon="page.icon" color="inherit" />
          {{ page.title }}
        </router-link>
      </nav>
      <div class="pico toolbar">
        <div role="group">
          <div class="sync-button active" @click="sync"
            :data-tooltip="syncing ? 'Gathering your Steam sales' : 'Get latest Steam sales'"
            :class="{ 'syncing': syncing }"
            :disabled="syncing">
            <span class="sync-indicator" v-if="syncing" aria-busy="true" />
            <sb-icon v-else icon="sync" color="inherit" />
            Sync{{ syncing ? 'ing' : '' }}
            <span class="sync-percentage" v-if="syncing">{{ `${(progress * 100).toLocaleString(undefined, {
              maximumFractionDigits: 0
              })}%` }}</span>
          </div>
          <div class="settings-button clickable" @click="showSettings = !showSettings">
            <sb-icon icon="settings" color="inherit" />
          </div>
        </div>
      </div>
    </header>
    <router-view></router-view>
    <Settings v-if="showSettings" @close="showSettings = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, inject } from 'vue'
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import Toolbar from '@/components/Toolbar.vue';
import Settings from '@/components/Settings.vue';


const pages = [{
  name: 'overview',
  title: 'Overview',
  icon: 'dashboard'
}, {
  name: 'world',
  title: 'World',
  icon: 'public'
}, {
  name: 'export',
  title: 'Export',
  icon: 'file_download'
}];

const route = useRoute();
const activePage = computed(() => {
  return route.name || '';
});

const showSettings = ref(false);
const progress = ref(1);
const syncing = computed(() => progress.value < 1);
const alert = inject('alert')

listen('sync-progress', (event) => {
  progress.value = parseFloat(event.payload);
});

async function sync() {
  if (progress.value < 1) return; // Prevent multiple syncs
  try {
    progress.value = 0;
    await invoke<string>("sync_command", {});
  } catch (e) {
    console.error("Sync failed:", e);
    alert(e)
  }
}
</script>

<style lang="scss" scoped>
.home-container {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: min-content 1fr;
  height: 100%;
  width: 100%;

  header {
    display: grid;
    grid-template-columns: 1fr 3fr 1fr;
    align-items: center;

    nav {
      display: inline-flex;
      align-items: center;
      gap: 4px;
      justify-self: center;
      grid-column: 2;
      border-radius: 8px;
      background-color: color-mix(in oklab, var(--pico-background-color), var(--pico-muted-border-color) 30%);
      padding: 8px;
    }

    [role="group"] {
      margin: 0;
    }

    [role="button"],
    .sync-button,
    .settings-button {
      display: inline-flex;
      align-items: center;
      gap: 4px;
      padding: 4px 8px;
      border-radius: 4px;
      color: var(--pico-muted-color);
      cursor: pointer;

      &:hover {
        background-color: var(--pico-muted-border-color);
      }

      &.active {
        background-color: var(--pico-primary);
        color: var(--pico-background-color);
      }
    }

    .toolbar {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      justify-self: end;

      [role="group"] {
        gap: 8px;
      }

      .sync-button {
        display: inline-grid;
        grid-template-columns: min-content max-content;
        align-items: center;
        justify-content: center;
        gap: 8px;
        font-size: 20px;
        // color: var(--pico-primary);
        padding: 4px 16px;
        transition: width 0.2s ease-in-out;
        width: 100px;

        &.syncing {
          grid-template-columns: min-content max-content 1fr;
          width: 180px;

          .sync-indicator {
            &::before {
              // filter: brightness(0.4);
              mix-blend-mode: multiply;
            }
          }
        }

        .sync-percentage {
          justify-self: end;
        }

        // &:hover {
        //     background: var(--pico-muted-border-color);
        // }
      }
    }
  }
}
</style>