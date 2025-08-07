<template>
    <sb-icon class="rotating"
        :size="size"
        :color="color" icon="progress_activity" />
</template>

<script setup>
import { ref } from 'vue'
import { parseColor } from '@/utils.js'

const props = defineProps({
    color: {
        type: String,
        default: "blue-500"
    },
    size: {
        type: String,
        default: "1em"
    }
})

const style = ref({
    'color': parseColor(props.color),
    'font-size': props.size
})

</script>

<style lang="scss" scoped>
.rotating {
    animation: rotating 1s linear infinite;
    @keyframes rotating {
        from {
            transform: rotate(0deg);
        }

        to {
            transform: rotate(360deg);
        }
    }
}

</style>