<template>
    <span :class="['material-symbols', { fill }]"
        :style="style">
        {{ icon }}
    </span>
</template>

<script setup lang="ts">
import { useAttrs, computed, ref } from 'vue';
import { parseColor } from '@/utils.js'

defineOptions({ name: 'sb-icon' });

const props = defineProps<{
    icon?: string;
    color?: string;
    fill?: boolean;
    size?: '1em';
}>();

const attrs = useAttrs();
const isClickable = computed(() => !!attrs.onClick);

const color = computed(() => {
    return parseColor(props.color)
});

const style = computed(() => {
    return {
        'color': color.value || '',
        'font-size': props.size,
        'cursor': isClickable.value ? 'pointer' : '',
    };
});

</script>

<style lang="scss" scoped>
span.material-symbols {
    font-family: 'Material Symbols Rounded';
    font-weight: normal;
    font-style: normal;
    /* Preferred icon size */
    display: inline-block;
    line-height: 1;
    text-transform: none;
    letter-spacing: normal;
    word-wrap: normal;
    white-space: nowrap;
    direction: ltr;
    --color: var(--pico-primary);
    color: var(--color);
    vertical-align: middle;

    &.fill {
        font-variation-settings: 'FILL' 1;
    }
}
</style>
