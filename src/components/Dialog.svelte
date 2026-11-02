<!-- A modal built on <dialog>: focus is trapped while open, Esc closes it, and focus goes back
     to whatever opened it. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  let {
    open = $bindable(false),
    title,
    width = 440,
    children,
    actions,
  }: { open?: boolean; title: string; width?: number; children: Snippet; actions?: Snippet } = $props();

  let el: HTMLDialogElement | undefined = $state();
  let opener: Element | null = null;

  $effect(() => {
    if (!el) return;
    if (open && !el.open) {
      opener = document.activeElement;
      el.showModal();
    } else if (!open && el.open) {
      el.close();
    }
  });

  function closed() {
    open = false;
    if (opener instanceof HTMLElement) opener.focus();
  }
</script>

<dialog bind:this={el} onclose={closed} style="--width: {width}px" aria-labelledby="dialog-title">
  {#if open}
    <header>
      <h2 id="dialog-title">{title}</h2>
      <button class="icon-btn" aria-label="Close" title="Close (Esc)" onclick={() => (open = false)}>
        <Icon name="close" />
      </button>
    </header>
    <div class="body">{@render children()}</div>
    {#if actions}
      <footer>{@render actions()}</footer>
    {/if}
  {/if}
</dialog>

<style>
  dialog {
    width: min(var(--width), calc(100vw - 2 * var(--gutter)));
    max-height: calc(100vh - 2 * var(--gutter));
    padding: 0;
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
    color: var(--text);
    box-shadow: var(--shadow-overlay);
  }
  dialog[open] {
    animation: enter var(--dur) var(--ease-out);
  }
  dialog::backdrop {
    background: rgb(20 24 31 / 0.45);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    padding: var(--s-4) var(--s-4) 0 var(--s-5);
  }
  h2 {
    font-size: var(--text-md);
  }
  .body {
    display: grid;
    gap: var(--s-4);
    padding: var(--s-4) var(--s-5);
  }
  footer {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: var(--s-2);
    padding: var(--s-4) var(--s-5);
    border-top: 1px solid var(--line);
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    dialog[open] {
      animation: none;
    }
  }
</style>
