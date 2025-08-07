<template>
  <div class="overview-container">
    <div class="widgets">
      <Revenue :currentSales="last30DaysSales" :previousSales="previous30DaysSales" />
      <UnitSold :currentSales="last30DaysSales" :previousSales="previous30DaysSales" />
      <TopPackages :currentSales="last30DaysSales" :previousSales="previous30DaysSales" />
      <!-- <RecentSales :currentSales="last30DaysSales" :previousSales="previous30DaysSales" /> -->
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import { useOptionsStore } from '@/stores/options.ts'

import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';

import { DateTime } from "luxon";

import Revenue from '../widgets/Revenue.vue';
import UnitSold from '../widgets/UnitSold.vue';
import TopPackages from '../widgets/TopPackages.vue';
import RecentSales from '../widgets/RecentSales.vue';

const options = useOptionsStore();
const period = computed(() => options.period);

watch(period, (newPeriod) => {
  get_detailed_sales();
});

listen("sync-data", (event) => {
  const results = event.payload
  if (results) {
    results.forEach((result) => {
      if (options.isInPeriod(result.date)) {
        last30DaysSales.value.push(result);
      } else if (options.isInPreviousPeriod(result.date)) {
        previous30DaysSales.value.push(result);
      }
    })
  }
});

const last30DaysSales = ref([]);
const previous30DaysSales = ref([]);

onMounted(() => {
  get_detailed_sales();
});

async function get_detailed_sales() {
  let fromDate = options.from;
  let toDate = options.to;
  options.loading = true;

  const promises = [];
  promises.push(invoke("get_detailed_sales_command", { fromDate, toDate }).then((sales: any) => {
    last30DaysSales.value = sales;
  }));

  if (fromDate && toDate) {
    fromDate = options.previousFrom;
    toDate = options.previousTo;
    promises.push(invoke("get_detailed_sales_command", { fromDate, toDate }).then((sales: any) => {
      previous30DaysSales.value = sales;
    }));
  } else {
    previous30DaysSales.value.splice(0)
  }

  await Promise.all(promises);
  options.loading = false;
}

</script>

<style lang="scss" scoped>
.overview-container {
  display: grid;
  // grid-template-columns: repeat(auto-fit, minmax(600px, 1fr));
  grid-template-columns: 1fr;
  grid-template-rows: 1fr;
  height: 100%;
  padding: 16px;
  container: widgets / size;

  .widgets {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr;
    grid-auto-rows: 1fr;
    gap: 16px;
    align-items: center;
    justify-items: center;
    height: 100%;
  }
}

@container widgets (width < 1000px) {
  .overview-container {
    .widgets {
      grid-template-columns: 1fr;
    }
  }
}
</style>