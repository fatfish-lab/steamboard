<template>
  <div class="pico decrypting-container">
    <div class="container">
      <sb-icon icon="security" size="64px" />
      <h1>Waiting for decryption</h1>
      <p>The system is requesting access to your keychain to decrypt your Steam API key. Please grant permission when
        prompted to continue.
      </p>
      <small aria-busy="true">Secure decryption in progress...</small>
      <div class="help">
        <h6>What's happening?</h6>
        <p>Your system keychain securely stores encrypted secrets. We need your permission to access and decrypt the
          required credentials.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject } from 'vue'
import { useRouter } from 'vue-router'
import { listen } from '@tauri-apps/api/event';

const alert = inject('alert')
const router = useRouter()

listen('denied-keyring-access', (event) => {
  alert(event.payload)
  router.push({ name: 'welcome' })
})

listen('decryption-failed', (event) => {
  alert(event.payload)
  router.push({ name: 'welcome' })
})
</script>

<style lang="scss" scoped>
.decrypting-container {
  display: grid;
  grid-template-columns: 1fr;
  height: 100%;
  align-items: center;
  justify-items: center;

  .container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    border-radius: 8px;
    background-color: color-mix(in oklab, var(--pico-background-color), var(--pico-muted-border-color) 30%);
    max-width: 600px;
    padding: 32px;
    gap: 8px;

    .help {
      background-color: var(--pico-muted-border-color);
      padding: 12px;
      border-radius: 8px;
      margin-top: 16px;
    }
  }
}
</style>