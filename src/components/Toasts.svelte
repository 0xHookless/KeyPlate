<script lang="ts">
  import { toasts } from "../lib/toast.svelte";
</script>

<div class="toasts" role="status" aria-live="polite">
  {#each toasts as t (t.id)}
    <p class="toast notice notice-{t.tone}">{t.text}</p>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: var(--gutter);
    bottom: var(--gutter);
    left: var(--gutter);
    z-index: 50;
    display: grid;
    justify-items: end;
    gap: var(--s-2);
    pointer-events: none;
  }
  .toast {
    max-width: 420px;
    box-shadow: var(--shadow-overlay);
    color: var(--text);
    animation: rise var(--dur) var(--ease-out);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .toast {
      animation: none;
    }
  }
</style>
