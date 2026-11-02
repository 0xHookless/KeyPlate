<!--
  The signature element: seed words as a numbered grid of stamped cells, like a steel backup plate.
  Words are hidden until revealed. `stamping` runs the unlock sequence (cells stamp in turn).
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    count = 12,
    words = null,
    revealed = false,
    stamping = false,
    compact = false,
    label = "Seed phrase",
    footer,
  }: {
    count?: number;
    words?: string[] | null;
    revealed?: boolean;
    stamping?: boolean;
    compact?: boolean;
    label?: string;
    footer?: Snippet;
  } = $props();

  const cells = $derived(Array.from({ length: count }, (_, i) => i));
  const columns = $derived(count > 12 ? "many" : "few");
</script>

<div class="plate" class:compact class:stamping data-columns={columns} role="group" aria-label={label}>
  <span class="rivet tl" aria-hidden="true"></span>
  <span class="rivet tr" aria-hidden="true"></span>
  <span class="rivet bl" aria-hidden="true"></span>
  <span class="rivet br" aria-hidden="true"></span>

  <ol class="cells">
    {#each cells as i (i)}
      <li class="cell" style="--i: {i}">
        <span class="num">{String(i + 1).padStart(2, "0")}</span>
        {#if revealed && words && words[i]}
          <span class="word">{words[i]}</span>
        {:else}
          <span class="blank" aria-label="hidden word"></span>
        {/if}
      </li>
    {/each}
  </ol>

  {#if footer}
    <div class="edge">{@render footer()}</div>
  {/if}
</div>

<style>
  .plate {
    position: relative;
    padding: var(--s-5) var(--s-5) var(--s-4);
    border: 1px solid var(--plate-edge);
    border-radius: var(--radius-lg);
    background:
      repeating-linear-gradient(90deg, transparent 0 3px, rgb(255 255 255 / 0.05) 3px 4px),
      var(--plate-bed);
  }
  .compact {
    padding: var(--s-4) var(--s-4) var(--s-3);
  }

  .rivet {
    position: absolute;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--rivet);
    box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.35);
  }
  .tl { top: 8px; left: 8px; }
  .tr { top: 8px; right: 8px; }
  .bl { bottom: 8px; left: 8px; }
  .br { bottom: 8px; right: 8px; }

  .cells {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--s-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  [data-columns="many"] .cells {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
  @media (min-width: 900px) {
    [data-columns="few"] .cells {
      grid-template-columns: repeat(4, minmax(0, 1fr));
    }
  }
  /* Revealed words need room on a phone; blank compact plates keep three columns. */
  @media (max-width: 520px) {
    .plate:not(.compact) .cells {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  .cell {
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
    min-width: 0;
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--cell-edge);
    border-radius: var(--radius-sm);
    background: var(--cell);
    box-shadow:
      inset 0 1px 0 var(--cell-highlight),
      inset 0 -1px 0 var(--cell-shade);
    font-family: var(--font-mono);
  }
  .compact .cell {
    padding: 6px var(--s-2);
  }

  .num {
    flex: none;
    font-size: var(--text-xs);
    color: var(--text-3);
    font-variant-numeric: tabular-nums;
  }
  .word {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: var(--text-sm);
    font-weight: 500;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text);
    animation: appear var(--dur-fast) var(--ease-out);
  }
  .blank {
    flex: 1;
    align-self: center;
    height: 6px;
    max-width: 64px;
    border-radius: 3px;
    background: repeating-linear-gradient(90deg, var(--line-strong) 0 6px, transparent 6px 9px);
    opacity: 0.7;
  }

  .edge {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-2) var(--s-4);
    margin-top: var(--s-4);
    padding-top: var(--s-3);
    border-top: 1px dashed var(--plate-edge);
    font-size: var(--text-sm);
    color: var(--text-2);
  }

  /* The one orchestrated moment: during unlock, cells stamp in turn. About 700 ms per pass. */
  .stamping .cell {
    animation: stamp 1400ms var(--ease-out) infinite;
    animation-delay: calc(var(--i) * 55ms);
  }
  .stamping .num {
    animation: num 1400ms var(--ease-out) infinite;
    animation-delay: calc(var(--i) * 55ms);
  }

  @keyframes stamp {
    0%, 30%, 100% {
      transform: none;
      box-shadow: inset 0 1px 0 var(--cell-highlight), inset 0 -1px 0 var(--cell-shade);
    }
    12% {
      transform: translateY(1px) scale(0.985);
      box-shadow: inset 0 2px 3px var(--cell-shade);
      border-color: var(--accent);
    }
  }
  @keyframes num {
    0%, 30%, 100% { color: var(--text-3); }
    12% { color: var(--accent); }
  }
  @keyframes appear {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @media (prefers-reduced-motion: reduce) {
    .stamping .cell {
      animation: pulse 1600ms ease-in-out infinite;
      animation-delay: 0ms;
    }
    .stamping .num {
      animation: none;
    }
    .word {
      animation: none;
    }
    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.55; }
    }
  }
</style>
