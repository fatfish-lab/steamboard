<template>
  <div class="alert-container">
    <h3 class="title">{{ title }}</h3>
    <sb-icon icon="close" @click="$emit('close')" />
    <p>{{ message }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
const props = defineProps({
  title: String,
  message: String
})
</script>

<style lang="scss" scoped>
.alert-container {
  display: grid;
  grid-template-columns: 1fr max-content;
  grid-template-rows: repeat(2, min-content);
  padding: 16px;
  background-color: var(--pico-color-zinc-850);
  border-radius: 8px;

  .title {
    color: var(--pico-color-red-350);
  }

  p {
    grid-column: 1 / -1;
  }
}

</style>