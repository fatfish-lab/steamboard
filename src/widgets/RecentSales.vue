<template>
<Widget
    class="recent-sales-widget"
    title="Recent Sales"
    icon="shopping_cart">
    <table>
        <thead>
            <tr>
                <th scope="col">
                    <sb-icon icon="calendar_today" />
                    Date
                </th>
                <th scope="col">
                    <sb-icon icon="videogame_asset" />
                    Game
                </th>
                <th scope="col">
                    <sb-icon icon="attach_money" />
                    Revenue
                </th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="sale in recentSales" :key="sale.id">
                <td>{{ formatDate(sale.date, 'EEE dd MMMM') }}</td>
                <td>{{ sale.game }}</td>
                <td>${{ sale.revenue.toLocaleString(undefined, { style: "currency", currency: "USD" }) }}</td>
            </tr>
        </tbody>
    </table>
</Widget>
</template>

<script setup>
import { ref, computed } from 'vue'
import Widget from '@/components/Widget.vue'
import { formatDate } from '@/utils.js'

const props = defineProps({
    currentSales: Array,
    previousSales: Array
});

const recentSales = computed(() => {
    const sales = props.currentSales.reduce((dates, sale) => {
        const date = sale.date;
        const game = sale.package_name || 'Unknown';
        const id = `${date}-${game}`;
        if (!dates[id]) {
            dates[id] = {
                id,
                appid: sale.primary_appid,
                date,
                game,
                revenue: 0
            };
        }
        dates[id].revenue += sale.net_sales_usd || 0;
        return dates;
    }, {});

    return Object.values(sales).filter(sale => sale.revenue > 0).sort((a, b) => new Date(b.date) - new Date(a.date)).slice(0, 30);
});

</script>

<style lang="scss" scoped>
.recent-sales-widget {
    grid-column: 1 / -1;
    th {
        >span {
            line-height: 0;
        }
    }

}
</style>