<template>
    <Widget
        title="Unit sold"
        icon="toys_and_games"
        :trend="trend"
        :graph="graph">
        <template #header>
            {{ totalUnitsSold.toLocaleString() }}
        </template>
    </Widget>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useOptionsStore } from '@/stores/options.ts'
import Widget from '@/components/Widget.vue'

const props = defineProps({
    currentSales: Array,
    previousSales: Array,
});

const options = useOptionsStore();
const saleType = computed(() => options.salesType);

const saleProperty = computed(() => {
    return saleType.value === 'gross' ? 'gross_units_sold' : 'net_units_sold';
});

const totalUnitsSold = computed(() => {
    return props.currentSales.reduce((acc, sale) => acc + (sale[saleProperty.value] || 0), 0);
});

const totalPreviousUnitsSold = computed(() => {
    return props.previousSales.reduce((acc, sale) => acc + (sale[saleProperty.value] || 0), 0);
});

const trend = computed(() => {
    if (props.previousSales.length === 0) return null;
    if (totalPreviousUnitsSold.value === 0) return 0;
    return ((totalUnitsSold.value - totalPreviousUnitsSold.value) / totalPreviousUnitsSold.value * 100).toFixed(1);
});

const graph = computed(() => {
    const dates = props.currentSales.reduce((dates, sale) => {
        const date = sale.date
        if (!dates[date]) {
            dates[date] = 0;
        }
        dates[date] += sale[saleProperty.value] || 0;
        return dates;
    }, {});
    return Object.entries(dates)
        .map(([date, value]) => ({ date, value, unit: 'units' }))
        .sort((a, b) => new Date(a.date) - new Date(b.date));
});
</script>

<style lang="scss" scoped>

</style>