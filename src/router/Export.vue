<template>
  <div class="export-container">
    <sb-widget title="Clipboard" icon="content_copy">
      <section>
        <p>Copy your CSV data to your clipboard</p>
        <button @click="copyToClipboard">
          <sb-icon v-if="copied" icon="done" color="inherit" />
          <sb-icon v-else icon="content_copy" color="inherit" />
          <span v-if="copied">Copied !</span>
          <span v-else>Copy to clipboard</span>
        </button>
      </section>
    </sb-widget>
    <sb-widget title="CSV file" icon="csv">
      <section>
        <p>Get your sales data into a CSV file</p>
        <button @click="exportToCSV">
          Export to CSV
        </button>
      </section>
    </sb-widget>
    <sb-hint class="hint">You can export your Steam sales data. Choose the format you prefer</sb-hint>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core';
import { dirname } from '@tauri-apps/api/path';
import { useOptionsStore } from '../stores/options';
import { save } from '@tauri-apps/plugin-dialog';
import { DateTime } from 'luxon';

const copied = ref(false);
// Prints the chosen path
const options = useOptionsStore();

async function copyToClipboard() {
  copied.value = false;
  await invoke("copy_to_clipboard_command", { fromDate: options.from, toDate: options.to, delimiter: ";" });
  copied.value = true;
  setTimeout(() => {
    copied.value = false;
  }, 3000);
}

async function exportToCSV() {
  let defaultPath = `steam-sales-${DateTime.now().toFormat("yyyy-MM-dd")}.csv`;
  if (options.fromDateTime && options.toDateTime) {
    defaultPath = `steam-sales-${options.fromDateTime.toFormat("yyyy-MM-dd")}_${options.toDateTime.toFormat("yyyy-MM-dd")}.csv`;
  }

  const csvPath = await save({
    title: "Where do you want to save your CSV file?",
    defaultPath,
    filters: [
      {
        name: 'CSV files',
        extensions: ['csv'],
      },
    ],
  });

  await invoke("export_csv_command", { path: csvPath, fromDate: options.from, toDate: options.to, delimiter: ";" });

  const folders = await dirname(csvPath);
  invoke('open_command', { path: folders })
}
// No props, data, computed, or methods currently needed
</script>

<style lang="scss" scoped>
.export-container {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
  grid-template-rows: min-content min-content 1fr;
  gap: 16px;
  align-items: center;
  justify-items: center;
  height: 100%;
  padding: 32px 16px;

  >header {
    grid-column: 1 / -1;

  }

  section {
    width: 100%;

    .success-copy {
      color: var(--pico-color-green-300);
    }

    button {
      width: 100%;
    }
  }

  .hint {
    grid-column: 1 / -1;
    grid-row: -1;
    margin-top: auto;
    justify-self: start;
  }
}
</style>