<template>
  <div class="pico toolbar">
    <details id="period" class="dropdown">
      <summary role="button" class="secondary">
        {{ selectedPeriod?.label || 'Select a period' }}
      </summary>
      <ul>
        <li v-for="period in options.periods" :key="period.value">
          <label>
            <input type="radio" name="interval"
              :value="period.value"
              :checked="options.period == period.value"
              @click="options.period = period.value" />
            {{ period.label }}
          </label>
        </li>
      </ul>
    </details>
    <details id="salesType" class="dropdown">
      <summary role="button" class="secondary">
        <span>{{ capitalize(options.salesType) }}</span>
      </summary>
      <ul>
        <li>
          <label>
            <input type="radio" name="salesType"
              value="gross"
              :checked="options.salesType == 'gross'"
              @click="options.salesType = 'gross'" />
            Gross
          </label>
        </li>
        <li>
          <label>
            <input type="radio" name="salesType"
              value="net"
              :checked="options.salesType == 'net'"
              @click="options.salesType = 'net'" />
            Net
          </label>
        </li>
      </ul>
    </details>
    <span v-if="isLoading" aria-busy="true"></span>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { capitalize } from '../utils.js';
import { useOptionsStore } from '../stores/options.ts';
const props = defineProps({

})

const options = useOptionsStore();
const isLoading = computed(() => options.isLoading);
const selectedPeriod = computed(() => {
  return options.periods.find(period => period.value == options.period);
});

</script>

<style lang="scss" scoped>
.toolbar {
  display: flex;
  justify-content: flex-start;
  width: 100%;
  gap: 16px;

  details {
    margin: 0;

    summary[role="button"] {
      display: inline-flex;
      white-space: nowrap;
    }

    li {
      label {
        margin-bottom: 0;
      }
    }
  }
}
</style>