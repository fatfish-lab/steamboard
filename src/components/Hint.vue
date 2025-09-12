<template>
  <span class="hint-container" :class="{ 'has-tooltip': tooltip }" v-tooltip="tooltip">
    <sb-icon icon="help" color="violet-400" fill />
    <slot>{{ text }}</slot>
  </span>
</template>

<script setup lang="ts">
  import { defineProps } from 'vue'
  import vTooltip from '@/directives/vTooltip.vue'

  const props = defineProps<{
    text?: string
    tooltip?: string
  }>()
</script>

<style lang="scss" scoped>
  span.hint-container {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--pico-color-violet-400);

    &.has-tooltip {
      cursor: default;
    }
  }
</style>
