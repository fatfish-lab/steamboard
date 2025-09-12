<template></template>
<script lang="ts">
  import { computePosition, offset, flip, shift, autoUpdate } from '@floating-ui/dom';

  declare global {
    interface HTMLElement {
      _tooltipCleanup?: () => void;
    }
  }

  const vTooltip = {
    mounted(el: HTMLElement, binding: { value: string }) {
      if (!binding.value) return;

      let tooltipEl: HTMLDivElement | null = null;
      let cleanup: (() => void) | null = null;

      function showTooltip() {
        if (tooltipEl) return;
        tooltipEl = document.createElement('div');
        tooltipEl.textContent = binding.value;
        tooltipEl.style.position = 'absolute';
        tooltipEl.style.background = '#333';
        tooltipEl.style.color = '#fff';
        tooltipEl.style.padding = '4px 8px';
        tooltipEl.style.borderRadius = '4px';
        tooltipEl.style.zIndex = '9999';
        document.body.appendChild(tooltipEl);
        cleanup = autoUpdate(el, tooltipEl, () => {
          computePosition(el, tooltipEl!, {
            placement: 'top',
            middleware: [offset(8), flip(), shift({ padding: 5 })],
          }).then(({ x, y }) => {
            tooltipEl!.style.left = `${x}px`;
            tooltipEl!.style.top = `${y}px`;
          });
        });
      }

      function hideTooltip() {
        if (tooltipEl) {
          document.body.removeChild(tooltipEl);
          tooltipEl = null;
        }
        if (cleanup) {
          cleanup();
          cleanup = null;
        }
      }

      el.addEventListener('mouseenter', showTooltip);
      el.addEventListener('mouseleave', hideTooltip);
      el._tooltipCleanup = hideTooltip;
    },
    beforeUnmount(el) {
      el._tooltipCleanup && el._tooltipCleanup();
    },
  };

  export default vTooltip;
</script>

<style>
  /* Add any additional styles here */
</style>