<template>
  <div class="widget-container pico" :class="{ 'with-graph': graph != null, 'big-graph': graph && graph.length > 200 }"
    :style="{ 'grid-column': `span ${span || 1}` }">
    <h4 class="widget-title">{{ title }}</h4>
    <sb-icon v-if="icon" class="icon" :icon="icon" :color="color" size="42px" fill />
    <div class="widget-content">
      <h2>
        <slot name="header"></slot>
      </h2>
      <slot></slot>
      <small v-if="trend != null"
        data-tooltip="Compared to previous period"
        data-placement="right"
        class="trend"
        :class="{ 'negative': !trendPositive }">
        {{ trendPositive ? '+' : '' }}{{ trend }}%
      </small>
    </div>
    <div class="graph" v-if="graph"
      :style="{ '--graph-length': filteredGraph.length, '--graph-max': maxGraph }">
      <div class="bar" v-for="(data, index) in filteredGraph" :key="data.date"
        :data-tooltip="`${formatDate(data.date)} ${data.end ? `> ${formatDate(data.end)} -` : '-'} ${data.value}${data.unit ? ` ${data.unit}` : ''}`"
        :data-placement="index < (filteredGraph.length / 2) ? 'top-start' : 'top-end'"
        :style="{ '--bar-value': data.value }"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { formatDate } from '@/utils.js'

type GraphData = {
  value: number;
  date: string;
  end?: string; // Optional end date for ranges
  unit?: string;
}

const props = defineProps<{
  title: String,
  icon?: String,
  color?: String,
  trend?: Number,
  span?: Number,
  graph?: Array<GraphData>
}>();

const trendPositive = computed(() => {
  return props.trend > 0;
});

const filteredGraph = computed(() => {
  if (!props.graph || props.graph.length <= 365) return props.graph;
  const packSize = Math.ceil(props.graph.length / 365);
  return props.graph.reduce((acc, data, index) => {
    if (index % packSize === 0) {
      if (acc.length > 0) {
        const lastPack = acc[acc.length - 1];
        lastPack.end = data.date;
      }
      acc.push(data);
    } else {
      acc[acc.length - 1].value = Math.round((acc[acc.length - 1].value + data.value) * 100) / 100; // Sum the values of the same pack
    }
    return acc;
  }, [] as GraphData[]);

});

const maxGraph = computed(() => {
  if (!props.graph || props.graph.length === 0) return 1;
  return Math.max(...filteredGraph.value.map(data => data.value));
});
</script>

<style lang="scss" scoped>
.widget-container {
  display: grid;
  grid-template-columns: 1fr max-content;
  grid-template-rows: min-content 1fr;
  gap: 8px;
  border-radius: 8px;
  padding: 32px;
  width: 100%;
  height: 100%;
  cursor: default;
  position: relative;
  --bg-color: color-mix(in oklab, var(--pico-background-color), var(--pico-muted-border-color) 30%);
  background-color: var(--bg-color);
  --bar-color: var(--pico-muted-border-color);
  overflow: clip;

  &.with-graph {
    grid-template-rows: min-content auto 1fr;

    &.big-graph {
      .graph {
        gap: 0;
      }
    }

    .graph {
      display: grid;
      grid-column: 1 / -1;
      grid-template-columns: repeat(var(--graph-length), 1fr);
      align-items: end;
      gap: 2px;
      height: 100%;
      width: 100%;
      position: relative;

      &::after {
        content: '';
        display: block;
        position: absolute;
        bottom: 0;
        left: 0;
        height: 100%;
        width: 100%;
        background: linear-gradient(to top, var(--bg-color) 0%, transparent);
        opacity: 0.5;
      }

      .bar {
        border-top-left-radius: 4px;
        border-top-right-radius: 4px;
        height: 100px;
        background-color: var(--bar-color);
        height: calc((var(--bar-value) / var(--graph-max)) * 100%);
        min-width: 2px;

        &:first-of-type {
          border-bottom-left-radius: 4px;
        }

        &:last-of-type {
          border-bottom-right-radius: 4px;
        }
      }
    }
  }

  &:hover {
    background-color: var(--pico-muted-border-color);
    --bar-color: var(--pico-primary-border);

    .graph {
      &::after {
        display: none;
      }
    }
  }

  .widget-content {
    grid-column: 1 / -1;
    display: inline-flex;
    align-items: baseline;
    gap: 8px;
  }

  h4,
  h2 {
    margin: 0;
  }

  .trend {
    color: var(--pico-color-green-300);

    &.negative {
      color: var(--pico-color-red-300);
    }
  }
}
</style>