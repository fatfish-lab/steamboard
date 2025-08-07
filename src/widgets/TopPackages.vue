<template>
    <Widget
        :title="`Top ${maximumPackages} packages`"
        description="Last 30 days"
        icon="leaderboard">
        <template #header>
            <div class="package" v-for="(pkg, p) in top5Packages" :key="pkg.name">
                <img :src="steamOgImage(pkg.primary_appid)" alt="">
                <h5 class="name">{{ pkg.name }}</h5>
                <sb-icon v-if="p == 0" icon="crown" size="1rem"/>
                <h4 class="sales">${{ pkg.sales.toLocaleString(undefined, { style: "currency", currency: "USD" }) }}</h4>
            </div>
        </template>
    </Widget>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useOptionsStore } from '@/stores/options.ts'
import { steamOgImage } from '@/utils.js'

import Widget from '@/components/Widget.vue'

const maximumPackages = 5;
const props = defineProps({
    currentSales: Array,
    previousSales: Array,
    salesType: String
});

const options = useOptionsStore();
const saleType = computed(() => options.salesType);

const saleProperty = computed(() => {
    return saleType.value === 'gross' ? 'gross_sales_usd' : 'net_sales_usd';
});

const packages = computed(() => {
    return props.currentSales.reduce((acc, sale) => {
        const packageName = sale.package_name || 'Unknown';
        if (!acc[packageName]) {
            acc[packageName] = {
                name: packageName,
                primary_appid: sale.primary_appid,
                sales: 0
            };
        }
        acc[packageName].sales += sale[saleProperty.value] || 0;
        return acc;
    }, {});
});

const top5Packages = computed(() => {
    const sortedPackages = Object.values(packages.value).sort((a, b) => b.sales - a.sales);
    return sortedPackages.slice(0, maximumPackages).filter(pkg => pkg.sales > 0).map(pkg => ({
        ...pkg,
        sales: Math.round(pkg.sales * 100) / 100 // Round to 2 decimal places
    }));
});


</script>

<style lang="scss" scoped>

.package {
    display: inline-flex;
    justify-content: flex-start;
    align-items: center;
    gap: 8px;
    width: 100%;

    img {
        border-radius: 4px;
    }

    h5 {
        margin: 0;
    }

    .sales {
        margin-left: auto;
        margin-bottom: 0;
    }
}

</style>