<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorText, onEvent, type EntryDetail, type EntryKind, type EntrySummary, type Status } from "../lib/api";
  import { kindLabel, plural } from "../lib/format";
  import { notify } from "../lib/toast.svelte";
  import Icon from "../components/Icon.svelte";
  import SeedPlate from "../components/SeedPlate.svelte";
  import PracticeDialog from "../components/PracticeDialog.svelte";
  import EntryView from "./EntryView.svelte";
  import EntryEditor from "./EntryEditor.svelte";
  import Settings from "./Settings.svelte";
  import RecoverySheet from "./RecoverySheet.svelte";

  let {
    status,
    justCreated,
    onStatus,
  }: { status: Status; justCreated: boolean; onStatus: (s: Status) => void } = $props();

  type Mode = "view" | "edit" | "settings" | "recovery";

  let entries = $state<EntrySummary[]>([]);
  let loaded = $state(false);
  let selectedId = $state<string | null>(null);
  let mode = $state<Mode>("view");
  let editKind = $state<EntryKind>("seed");
  let editing = $state<EntryDetail | null>(null);
  let query = $state("");
  let detailOpen = $state(false); // narrow screens: detail pane covers the list
  let showCreated = $state(false);
  let practiceOpen = $state(false);
  let practiceDismissed = $state(false);
  let detailEl: HTMLElement | undefined = $state();

  $effect.pre(() => {
    if (justCreated) showCreated = true;
  });

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return q ? entries.filter((e) => `${e.title} ${e.network ?? ""}`.toLowerCase().includes(q)) : entries;
  });
  const selected = $derived(entries.find((e) => e.id === selectedId) ?? null);
  const seedCount = $derived(entries.filter((e) => e.kind === "seed").length);

  const wide = () => !window.matchMedia("(max-width: 760px)").matches;

  async function refresh() {
    try {
      entries = await api.listEntries();
    } catch (e) {
      notify(errorText(e), "danger");
      return;
    }
    loaded = true;
    if (selectedId && !entries.some((e) => e.id === selectedId)) selectedId = null;
    if (!selectedId && entries.length && wide() && mode === "view") selectedId = entries[0].id;
  }

  onMount(() => {
    refresh();
    const off = onEvent("vault-changed", () => {
      refresh();
      notify("Updated with changes from another PC.");
    });
    return () => {
      off.then((f) => f());
    };
  });

  function show(next: Mode) {
    mode = next;
    detailOpen = true;
    queueMicrotask(() => {
      if (detailEl) detailEl.scrollTop = 0;
      detailEl?.focus({ preventScroll: true });
    });
  }

  function select(id: string) {
    selectedId = id;
    show("view");
  }

  function create(kind: EntryKind) {
    editKind = kind;
    editing = null;
    show("edit");
  }

  function edit(d: EntryDetail) {
    editKind = d.kind;
    editing = d;
    show("edit");
  }

  async function saved(s: EntrySummary) {
    editing = null;
    await refresh();
    selectedId = s.id;
    show("view");
    notify(`Saved “${s.title}”.`, "ok");
  }

  function cancelEdit() {
    editing = null;
    mode = "view";
    if (!selectedId) detailOpen = false;
  }

  function back() {
    detailOpen = false;
    if (mode !== "view") mode = "view";
  }

  async function lock() {
    onStatus(await api.lock());
  }
</script>

<div class="vault">
  <header class="bar">
    <p class="mark"><Icon name="plate" size={20} /> <span>Keyplate</span></p>
    <label class="search">
      <Icon name="search" size={16} />
      <span class="visually-hidden">Search by name</span>
      <input type="search" class="input" bind:value={query} placeholder="Search by name" />
    </label>
    <div class="tools">
      <button class="btn btn-primary" onclick={() => create("seed")} aria-label="New seed phrase" title="New seed phrase">
        <Icon name="plus" /> <span class="btn-text">New seed phrase</span>
      </button>
      <button class="btn" onclick={() => create("note")} aria-label="New note" title="New note">
        <Icon name="note" /> <span class="btn-text">New note</span>
      </button>
      <button class="icon-btn" class:active={mode === "settings"} onclick={() => show("settings")} aria-label="Settings" title="Settings">
        <Icon name="sliders" />
      </button>
      <button class="btn" onclick={lock} title="Lock now"><Icon name="lock" /> <span class="btn-text">Lock</span></button>
    </div>
  </header>

  {#if status.offline || showCreated || (status.practiceDue && !practiceDismissed)}
    <div class="banners">
      {#if status.offline}
        <p class="notice notice-caution">
          <Icon name="alert" />
          <span><strong>Opened the offline copy.</strong> Google Drive's folder couldn't be reached. Changes are saved here and reach Drive once it's back.</span>
        </p>
      {/if}
      {#if showCreated}
        <div class="notice notice-ok">
          <Icon name="check" />
          <span>
            <strong>Your vault is sealed.</strong> Print the recovery sheet now. It tells future-you how to restore this
            vault, and holds no secrets.
          </span>
          <span class="banner-actions">
            <button class="btn btn-sm" onclick={() => { showCreated = false; show("recovery"); }}>Open recovery sheet</button>
            <button class="icon-btn" aria-label="Dismiss" title="Dismiss" onclick={() => (showCreated = false)}><Icon name="close" size={16} /></button>
          </span>
        </div>
      {/if}
      {#if status.practiceDue && !practiceDismissed}
        <div class="notice notice-info">
          <Icon name="key" />
          <span><strong>Time for a practice unlock.</strong> Typing both passwords now and then keeps them from fading.</span>
          <span class="banner-actions">
            <button class="btn btn-sm" onclick={() => (practiceOpen = true)}>Practise now</button>
            <button class="icon-btn" aria-label="Remind me later" title="Remind me later" onclick={() => (practiceDismissed = true)}><Icon name="close" size={16} /></button>
          </span>
        </div>
      {/if}
    </div>
  {/if}

  <div class="split" class:detail-open={detailOpen}>
    <nav class="list" aria-label="Vault entries">
      {#if loaded}
        <p class="count">{plural(seedCount, "seed phrase")} · {plural(entries.length - seedCount, "note")}</p>
        {#if filtered.length}
          <ul>
            {#each filtered as e (e.id)}
              <li>
                <button
                  class="item"
                  class:selected={e.id === selectedId && mode === "view"}
                  aria-current={e.id === selectedId && mode === "view" ? "true" : undefined}
                  onclick={() => select(e.id)}
                >
                  <span class="item-icon" aria-hidden="true"><Icon name={e.kind === "seed" ? "plate" : "note"} size={18} /></span>
                  <span class="item-text">
                    <span class="item-title">{e.title}</span>
                    <span class="item-meta">
                      {e.kind === "seed" ? plural(e.wordCount, "word") : kindLabel(e.kind)}{e.network ? ` · ${e.network}` : ""}
                    </span>
                  </span>
                  {#if e.checksumValid === false}
                    <span class="flag" title="Not a valid BIP39 checksum"><Icon name="alert" size={16} /><span class="visually-hidden">Checksum not valid</span></span>
                  {/if}
                </button>
              </li>
            {/each}
          </ul>
        {:else if entries.length}
          <div class="list-empty">
            <p>Nothing is named “{query}”.</p>
            <button class="link-btn" onclick={() => (query = "")}>Clear search</button>
          </div>
        {/if}
      {/if}
    </nav>

    <section class="detail" bind:this={detailEl} tabindex="-1" aria-label="Details">
      <button class="btn btn-quiet back" onclick={back}><Icon name="back" /> All entries</button>

      {#if mode === "settings"}
        <Settings {status} {onStatus} onRecovery={() => show("recovery")} />
      {:else if mode === "recovery"}
        <RecoverySheet />
      {:else if mode === "edit"}
        {#key editing?.id ?? `new-${editKind}`}
          <EntryEditor initial={editing} kind={editKind} onSaved={saved} onCancel={cancelEdit} />
        {/key}
      {:else if selected}
        {#key selected.id}
          <EntryView entry={selected} onEdit={edit} onDeleted={() => { selectedId = null; detailOpen = false; refresh(); }} />
        {/key}
      {:else if loaded && entries.length === 0}
        <div class="empty">
          <SeedPlate count={12} compact label="Empty plate" />
          <h2>Your vault is empty</h2>
          <p>
            Add your first seed phrase. It's sealed on this PC before Google Drive syncs anything, and Keyplate checks
            the words as you type.
          </p>
          <div class="empty-actions">
            <button class="btn btn-primary" onclick={() => create("seed")}><Icon name="plus" /> Add a seed phrase</button>
            <button class="btn" onclick={() => create("note")}>Write a note instead</button>
          </div>
        </div>
      {:else if loaded}
        <p class="hint pick">Choose an entry to see it.</p>
      {/if}
    </section>
  </div>
</div>

<PracticeDialog bind:open={practiceOpen} onDone={() => (practiceDismissed = true)} />

<style>
  .vault {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: var(--s-4);
    padding: var(--s-3) var(--s-5);
    border-bottom: 1px solid var(--line);
    background: var(--surface-raised);
  }
  .mark {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    width: 248px;
    flex: none;
    font-weight: 700;
    color: var(--accent);
  }
  .mark span {
    color: var(--text);
  }
  .search {
    position: relative;
    flex: 1;
    max-width: 360px;
    color: var(--text-3);
  }
  .search :global(svg) {
    position: absolute;
    top: 12px;
    left: 12px;
  }
  .search .input {
    padding-left: 36px;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-left: auto;
  }
  .icon-btn.active {
    background: var(--surface-sunk);
    color: var(--text);
  }

  .banners {
    display: grid;
    gap: var(--s-2);
    padding: var(--s-3) var(--s-5) 0;
  }
  .banners .notice {
    align-items: center;
  }
  .banners .notice > span:first-of-type {
    flex: 1;
  }
  .banner-actions {
    display: flex;
    align-items: center;
    gap: var(--s-1);
  }

  .split {
    display: grid;
    flex: 1;
    grid-template-columns: 300px minmax(0, 1fr);
    min-height: 0;
  }
  .list {
    overflow-y: auto;
    padding: var(--s-4) var(--s-3);
    border-right: 1px solid var(--line);
  }
  .count {
    padding: 0 var(--s-3) var(--s-3);
    font-size: var(--text-xs);
    color: var(--text-3);
  }
  .list ul {
    display: grid;
    gap: 2px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .item {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    width: 100%;
    min-height: 56px;
    padding: var(--s-2) var(--s-3);
    border: 0;
    border-radius: var(--radius);
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease-out);
  }
  .item:hover {
    background: var(--surface-sunk);
  }
  .item.selected {
    background: var(--accent-tint);
    box-shadow: inset 3px 0 0 var(--accent);
  }
  .item-icon {
    flex: none;
    color: var(--text-3);
  }
  .selected .item-icon {
    color: var(--accent);
  }
  .item-text {
    display: grid;
    min-width: 0;
    flex: 1;
  }
  .item-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 600;
  }
  .item-meta {
    font-size: var(--text-sm);
    color: var(--text-3);
  }
  .flag {
    flex: none;
    color: var(--caution);
  }
  .list-empty {
    display: grid;
    gap: var(--s-2);
    justify-items: start;
    padding: var(--s-3);
    font-size: var(--text-sm);
    color: var(--text-2);
  }

  .detail {
    overflow-y: auto;
    padding: var(--s-6) var(--s-7) var(--s-8);
  }
  .detail:focus {
    outline: none;
  }
  .back {
    display: none;
    margin: 0 0 var(--s-4) calc(-1 * var(--s-3));
  }
  .empty {
    display: grid;
    gap: var(--s-4);
    max-width: 520px;
  }
  .empty h2 {
    margin-top: var(--s-3);
    font-size: var(--text-lg);
  }
  .empty p {
    color: var(--text-2);
  }
  .empty-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-3);
  }
  .pick {
    padding-top: var(--s-4);
  }

  @media (max-width: 1080px) {
    .mark {
      width: auto;
    }
    .tools .btn-text {
      display: none;
    }
  }

  @media (max-width: 760px) {
    .bar {
      flex-wrap: wrap;
      gap: var(--s-2) var(--s-3);
      padding: var(--s-2) var(--gutter);
    }
    .search {
      order: 3;
      flex-basis: 100%;
      max-width: none;
    }
    .banners {
      padding: var(--s-3) var(--gutter) 0;
    }
    .banners .notice {
      flex-wrap: wrap;
    }
    .split {
      grid-template-columns: minmax(0, 1fr);
    }
    .list {
      border-right: 0;
      padding: var(--s-3) var(--s-2);
    }
    .detail {
      display: none;
      padding: var(--s-4) var(--gutter) var(--s-7);
    }
    .detail-open .list {
      display: none;
    }
    .detail-open .detail {
      display: block;
      overflow-anchor: none;
    }
    .detail-open .back {
      display: inline-flex;
    }
  }

  /* Empty vault on a narrow screen: the call to action lives in the detail pane. */
  @media (max-width: 760px) {
    .split:not(.detail-open):has(.empty) .detail {
      display: block;
    }
    .split:not(.detail-open):has(.empty) .list {
      display: none;
    }
  }

  @media print {
    .bar,
    .banners,
    .list,
    .back {
      display: none !important;
    }
    .vault {
      display: block;
      height: auto;
    }
    .split {
      display: block;
    }
    .detail {
      overflow: visible;
      padding: 0;
    }
  }
</style>
