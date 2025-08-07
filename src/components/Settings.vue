<template>
  <div class="pico settings-container">
    <dialog open>
      <article>
        <header>
          <h4>Settings</h4>
        </header>
        <form @submit.prevent>
          <label for="steam-api-key">Update your Steam API Key</label>
          <input
            v-model="settings.steam_api_key"
            type="password"
            id="steam-api-key"
            placeholder="AZDB5F..." />
          <label for="interval">Auto sync interval</label>
          <details id="interval" class="dropdown">
            <summary role="button" class="secondary">
              <span v-if="selectedInterval">Every {{ selectedInterval.label }}</span>
              <span v-else>Select an interval</span>
            </summary>
            <ul>
              <li v-for="interval in availableIntervals" :key="interval.value">
                <label>
                  <input type="radio" name="interval"
                    :value="interval.value"
                    :checked="settings.poll_interval == interval.value"
                    @click="settings.poll_interval = interval.value" />
                  {{ interval.label }}
                </label>
              </li>
            </ul>
          </details>
          <label for="location">Steamboard folder</label>
          <button class="secondary" @click="openLocation">
            <sb-icon icon="folder" size="16px" color="inherit" />
            Locate Steamboard folder</button>
        </form>
        <small>You are using v{{ updater.version }}</small>
        <footer>
          <button class="contrast" @click="$emit('close')">
            Cancel
          </button>
          <button @click="settings.save(), $emit('close')">Confirm</button>
        </footer>
      </article>
    </dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useUpdaterStore } from '@/stores/updater.ts'
import { useSettingsStore } from '@/stores/settings.ts'

const settings = useSettingsStore()
const updater = useUpdaterStore()
const selectedInterval = computed(() => {
  return availableIntervals.find(interval => interval.value == settings.poll_interval);
})
const availableIntervals = [{
  value: 60,
  label: '1 minute'
}, {
  value: 300,
  label: '5 minutes'
}, {
  value: 600,
  label: '10 minutes'
}, {
  value: 3600,
  label: '1 hour'
  }, {
  value: 86400,
  label: '1 day'
}]


function openLocation() {
  invoke('open_location_command')
}
</script>

<style lang="scss" scoped>
.settings-container {
  dialog {
    article {
      overflow: visible;

      form {
        button {
          margin-bottom: var(--pico-spacing);
        }
      }

      li {
        label {
          margin-bottom: 0;
        }
      }

      //     height: 80%;

      //     footer {
      //         margin-top: auto;
      //     }
    }
  }
}
</style>