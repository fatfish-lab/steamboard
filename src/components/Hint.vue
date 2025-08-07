<template>
    <span class="hint-container">
        <sb-icon icon="help" color="violet-400" fill />
        <slot>{{ text }}</slot>
    </span>
</template>

<script setup lang="ts">

</script>

<style lang="scss" scoped>
span.hint-container {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--pico-color-violet-400);
}
</style>
