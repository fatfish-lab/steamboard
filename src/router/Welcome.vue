<template>
  <div class="pico welcome-container container" v-if="!loading">
    <div class="welcome-form">
      <hgroup>
        <h1>Welcome on</h1>
        <img src="@/assets/logo_title_white.svg" alt="Steamboard logo">
        <h5>Connect your Steam account to synchronize your sales data</h5>
      </hgroup>
      <form @submit.prevent="connect">
        <fieldset>
          <label>
            Steam API Key
            <input
              v-model="settings.steam_api_key"
              name="steam_api_key"
              placeholder="Steam API Key"
              autocomplete="password"
              type="password"
              autofocus />
            <small>This API key provides read-only access to your financial data, which remains all stored
              locally.</small>
          </label>
        </fieldset>
        <span class="error-message" v-if="connectError">
          <sb-icon icon="error" />
          {{ connectError }}
        </span>

        <input
          type="submit"
          value="Connect"
          :disabled="loading || !settings.steam_api_key" />
        <sb-hint class="clickable" @click="openHelp">Learn how to get your Steam Financial API key</sb-hint>
      </form>
      <!-- <div class="security-notice container">
                <sb-icon icon="security" color="green-50" />
                <h5 class="title">Your data is safe</h5>
                <p>This API key only grants <strong>read-only access</strong> to your financial information. All data
                    stays locally on your computer and is never transmitted to external servers.</p>
                <a class="clickable" @click.prevent="openPrivacyStatement">Read our privacy statement →</a>
            </div> -->
    </div>
  </div>
  <div class="pico welcome-container loading container" v-else>
    <span aria-busy="true"></span>
    <hgroup>
      <h4>Verifying API key</h4>
      <h5>Please wait while we validate your credentials...</h5>
    </hgroup>
  </div>
</template>

<script setup>
import { ref, inject } from 'vue'
import { useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings.ts'
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from '@tauri-apps/plugin-opener';

const alert = inject('alert')
const settings = useSettingsStore()
const router = useRouter()
const loading = ref(false)
const connectError = ref("")
const showHelp = ref(false)

async function connect() {
  loading.value = true

  try {
    await invoke("check_api_key_command", { "steam": settings.steam_api_key })
  } catch (error) {
    console.log("ERROR", error)
    connectError.value = "Invalid API key. Please check your key and try again"
    loading.value = false
    return
  }

  // TODO: Add catch(message_toaster) to handle errors
  await settings.save().catch(alert)

  invoke("sync_command")

  router.push({ name: 'overview' })
}

function openPrivacyStatement() {
  openUrl("https://steamboard.app/#privacy")
}

function openHelp() {
  openUrl("https://github.com/fatfish-lab/steamboard/wiki/")
}

</script>

<style lang="scss" scoped>
.welcome-container {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: 1fr;
  gap: 24px;
  align-content: center;
  justify-items: center;
  text-align: center;

  &.loading {
    grid-template-rows: min-content max-content;

    .loading-icon {
      width: 63px; // Adjust width to avoid rotational wiggle.
    }
  }

  .welcome-form {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;

    hgroup {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 8px;
      text-align: center;

      h1 {
        color: var(--pico-color-zinc-450);
      }
    }

    .error-message {
      display: inline-flex;
      align-items: center;
      gap: 4px;
      color: var(--pico-color-red-400);
    }

    .security-notice {
      display: grid;
      grid-template-columns: min-content 1fr;
      grid-template-rows: min-content;
      padding: 16px;
      border: 1px solid var(--pico-color-green-200);
      border-radius: 8px;
      background-color: color-mix(in oklab, var(--pico-color-green-500), transparent 90%);
      font-size: 0.9em;
      max-width: 800px;
      gap: 8px;
      align-items: center;
      margin-top: 64px;


      .title {
        color: var(--pico-color-green-50);
                margin: 0;
            }

            >a,
            >p {
                grid-column: 1 / -1;
                color: var(--pico-color-green-50);
                text-decoration-color: var(--pico-color-green-200);
            }
        }
    }
}
</style>
