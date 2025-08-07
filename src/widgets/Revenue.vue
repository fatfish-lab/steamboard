<template>
    <Widget
        title="Total Revenue"
        icon="attach_money"
        :trend="trend"
        :graph="graph">
        <template #header>
            {{ totalRevenue.toLocaleString(undefined, { style: "currency", currency: "USD" }) }}
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
    return saleType.value === 'gross' ? 'gross_sales_usd' : 'net_sales_usd';
});

const totalRevenue = computed(() => {
    return props.currentSales.reduce((acc, sale) => acc + (sale[saleProperty.value] || 0), 0);
});

const totalPreviousRevenue = computed(() => {
    return props.previousSales.reduce((acc, sale) => acc + (sale[saleProperty.value] || 0), 0);
});

const trend = computed(() => {
    if (props.previousSales.length === 0) return null;
    if (totalPreviousRevenue.value === 0) return 0;
    return (((totalRevenue.value - totalPreviousRevenue.value) / totalPreviousRevenue.value) * 100).toFixed(1);
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
    return Object.keys(dates).map(date => {
        return {
            date,
            value: Math.round(dates[date] * 100) / 100,
            unit: '$'
        };
    }).sort((a, b) => new Date(a.date) - new Date(b.date));
});
</script>

<style lang="scss" scoped>
</style>