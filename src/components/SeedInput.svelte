<!-- Typing a seed phrase into the plate. Pasting a whole phrase into any cell fills the cells
     from there; Space or Enter moves to the next cell. -->
<script lang="ts">
  let {
    words = $bindable([]),
    invalid = new Set<number>(),
    onchange,
    onpastecount,
  }: {
    words?: string[];
    invalid?: Set<number>;
    onchange?: () => void;
    onpastecount?: (count: number) => void;
  } = $props();

  let cells: HTMLInputElement[] = $state([]);
  const STANDARD = [12, 15, 18, 21, 24];

  function focusCell(i: number) {
    cells[Math.min(i, words.length - 1)]?.focus();
  }

  function keydown(e: KeyboardEvent, i: number) {
    if ((e.key === " " || e.key === "Enter") && words[i]?.trim()) {
      e.preventDefault();
      focusCell(i + 1);
    } else if (e.key === "Backspace" && !words[i] && i > 0) {
      e.preventDefault();
      focusCell(i - 1);
    }
  }

  /** Spreads several words across the cells, starting at cell `i`. */
  function distribute(i: number, text: string) {
    const parts = text.trim().split(/[\s,]+/).filter(Boolean).map((w) => w.toLowerCase());
    const trailingSpace = /\s$/.test(text);
    if (i === 0 && parts.length > 1 && STANDARD.includes(parts.length) && parts.length !== words.length) {
      onpastecount?.(parts.length);
      words = parts;
    } else {
      const next = [...words];
      parts.forEach((w, k) => {
        if (i + k < next.length) next[i + k] = w;
      });
      if (!parts.length) next[i] = "";
      words = next;
    }
    onchange?.();
    const jump = parts.length > 1 || trailingSpace ? parts.length : 0;
    if (jump) queueMicrotask(() => focusCell(i + jump));
  }

  function paste(e: ClipboardEvent, i: number) {
    const text = e.clipboardData?.getData("text") ?? "";
    if (!/\S\s+\S/.test(text.trim())) return;
    e.preventDefault();
    distribute(i, text);
  }

  // Typing a space (or any input method that inserts one) moves on to the next cell.
  function input(e: Event, i: number) {
    const value = (e.currentTarget as HTMLInputElement).value;
    if (/\s/.test(value)) distribute(i, value);
    else onchange?.();
  }
</script>

<ol class="grid" class:many={words.length > 12}>
  {#each words as _, i (i)}
    <li class="cell" class:bad={invalid.has(i)}>
      <label class="num mono" for="word-{i}">{String(i + 1).padStart(2, "0")}</label>
      <input
        bind:this={cells[i]}
        id="word-{i}"
        class="word mono"
        bind:value={words[i]}
        oninput={(e) => input(e, i)}
        onkeydown={(e) => keydown(e, i)}
        onpaste={(e) => paste(e, i)}
        autocomplete="off"
        autocapitalize="off"
        spellcheck="false"
        aria-label="Word {i + 1}"
        aria-invalid={invalid.has(i) ? "true" : undefined}
      />
    </li>
  {/each}
</ol>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--s-2);
    margin: 0;
    padding: var(--s-4);
    border: 1px solid var(--plate-edge);
    border-radius: var(--radius-lg);
    background: var(--plate-bed);
    list-style: none;
  }
  .many {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
  @media (max-width: 560px) {
    .grid,
    .many {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  .cell {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    min-width: 0;
    padding: 0 0 0 var(--s-3);
    border: 1px solid var(--cell-edge);
    border-radius: var(--radius-sm);
    background: var(--cell);
    box-shadow: inset 0 1px 0 var(--cell-highlight);
    transition: border-color var(--dur-fast) var(--ease-out);
  }
  .cell:focus-within {
    border-color: var(--accent);
    box-shadow: var(--focus-ring);
  }
  .cell.bad {
    border-color: var(--danger);
    background: var(--danger-tint);
  }
  .num {
    flex: none;
    font-size: var(--text-xs);
    color: var(--text-3);
  }
  .word {
    flex: 1;
    min-width: 0;
    height: 38px;
    padding: 0 var(--s-2) 0 0;
    border: 0;
    background: transparent;
    font-size: var(--text-sm);
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }
  .word:focus-visible {
    box-shadow: none;
  }
</style>
