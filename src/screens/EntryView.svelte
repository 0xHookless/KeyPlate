<!-- One entry. Secrets are fetched only when revealed and dropped again on hide, after a minute,
     or when the window is hidden. -->
<script lang="ts">
  import { onDestroy } from "svelte";
  import { api, errorText, type EntryDetail, type EntrySummary } from "../lib/api";
  import { ago, kindLabel, plural } from "../lib/format";
  import { notify } from "../lib/toast.svelte";
  import Icon from "../components/Icon.svelte";
  import SeedPlate from "../components/SeedPlate.svelte";
  import Dialog from "../components/Dialog.svelte";

  let {
    entry,
    onEdit,
    onDeleted,
  }: { entry: EntrySummary; onEdit: (d: EntryDetail) => void; onDeleted: () => void } = $props();

  const AUTO_HIDE_MS = 60_000;
  const HOLD_MS = 600;

  let detail = $state<EntryDetail | null>(null);
  let holding = $state(false);
  let byHold = false;
  let confirmDelete = $state(false);
  let deleting = $state(false);
  let hideTimer: ReturnType<typeof setTimeout> | undefined;
  let holdTimer: ReturnType<typeof setTimeout> | undefined;

  const isSeed = $derived(entry.kind === "seed");

  async function reveal() {
    detail = await api.revealEntry(entry.id);
    clearTimeout(hideTimer);
    hideTimer = setTimeout(hide, AUTO_HIDE_MS);
  }

  function hide() {
    detail = null;
    byHold = false;
    clearTimeout(hideTimer);
  }

  function toggle() {
    if (detail) hide();
    else reveal().catch((e) => notify(errorText(e), "danger"));
  }

  function holdStart(e: PointerEvent) {
    if (detail || e.button !== 0) return;
    holding = true;
    holdTimer = setTimeout(async () => {
      byHold = true;
      await reveal();
    }, HOLD_MS);
  }

  function holdEnd() {
    holding = false;
    clearTimeout(holdTimer);
    if (byHold) hide();
  }

  async function copy(field: "phrase" | "passphrase" | "body") {
    try {
      const seconds = await api.copyField(entry.id, field);
      notify(`Copied. The clipboard clears in ${seconds} seconds and stays out of clipboard history.`, "ok");
    } catch (e) {
      notify(errorText(e), "danger");
    }
  }

  async function edit() {
    try {
      onEdit(detail ?? (await api.revealEntry(entry.id)));
    } catch (e) {
      notify(errorText(e), "danger");
    }
  }

  async function remove() {
    deleting = true;
    try {
      await api.deleteEntry(entry.id);
      confirmDelete = false;
      notify(`Deleted “${entry.title}”.`);
      onDeleted();
    } catch (e) {
      notify(errorText(e), "danger");
    } finally {
      deleting = false;
    }
  }

  function visibility() {
    if (document.hidden) hide();
  }
  document.addEventListener("visibilitychange", visibility);
  onDestroy(() => {
    document.removeEventListener("visibilitychange", visibility);
    clearTimeout(hideTimer);
    clearTimeout(holdTimer);
  });
</script>

<article class="view">
  <header>
    <div class="titles">
      <h2>{entry.title}</h2>
      <p class="meta">
        {[kindLabel(entry.kind), entry.network, isSeed && plural(entry.wordCount, "word"), `Updated ${ago(entry.updated)}`]
          .filter(Boolean)
          .join(" · ")}
      </p>
    </div>
    <div class="tools">
      <button class="btn btn-sm" onclick={edit}><Icon name="edit" size={16} /> Edit</button>
      <button class="btn btn-sm btn-quiet danger" onclick={() => (confirmDelete = true)}><Icon name="trash" size={16} /> Delete</button>
    </div>
  </header>

  {#if isSeed}
    <!-- Press and hold the plate to peek; the Reveal button does the same for keyboard users. -->
    <div
      class="hold"
      class:holding
      onpointerdown={holdStart}
      onpointerup={holdEnd}
      onpointerleave={holdEnd}
      onpointercancel={holdEnd}
      role="presentation"
    >
      <SeedPlate count={entry.wordCount} words={detail?.words ?? null} revealed={!!detail} label="{entry.title} seed phrase">
        {#snippet footer()}
          <span class="check">
            {#if entry.checksumValid}
              <span class="ok"><Icon name="check" size={16} /> BIP39 checksum valid</span>
            {:else if entry.checksumValid === false}
              <span class="caution"><Icon name="alert" size={16} /> Not a valid BIP39 checksum</span>
            {/if}
          </span>
          <span class="plate-actions">
            <button class="btn btn-sm" aria-pressed={!!detail} onclick={toggle}>
              <Icon name={detail ? "eye-off" : "eye"} size={16} />
              {detail ? "Hide words" : "Reveal words"}
            </button>
            <button class="btn btn-sm" onclick={() => copy("phrase")}><Icon name="copy" size={16} /> Copy phrase</button>
          </span>
        {/snippet}
      </SeedPlate>
      <p class="hold-hint">{detail ? "Words hide again after a minute." : "Or press and hold the plate to peek."}</p>
    </div>

    {#if entry.hasPassphrase}
      <section class="row">
        <div>
          <h3>BIP39 passphrase</h3>
          <p class="secret mono">{detail ? detail.bip39Passphrase : "••••••••••••"}</p>
        </div>
        <button class="btn btn-sm" onclick={() => copy("passphrase")}><Icon name="copy" size={16} /> Copy</button>
      </section>
    {/if}
  {/if}

  {#if entry.hasBody || !isSeed}
    <section class="notes">
      <div class="notes-head">
        <h3>{isSeed ? "Notes" : "Text"}</h3>
        <span class="plate-actions">
          {#if !isSeed}
            <button class="btn btn-sm" aria-pressed={!!detail} onclick={toggle}>
              <Icon name={detail ? "eye-off" : "eye"} size={16} />
              {detail ? "Hide" : "Reveal"}
            </button>
          {/if}
          <button class="btn btn-sm" onclick={() => copy("body")}><Icon name="copy" size={16} /> Copy</button>
        </span>
      </div>
      {#if detail}
        <p class="body">{detail.body}</p>
      {:else}
        <div class="redacted" aria-label="Hidden until revealed">
          <span style="width: 92%"></span><span style="width: 78%"></span><span style="width: 54%"></span>
        </div>
      {/if}
    </section>
  {/if}
</article>

<Dialog bind:open={confirmDelete} title="Delete “{entry.title}”?">
  <p>
    It disappears from this PC now, and from your other PCs once Google Drive syncs. Earlier copies stay in the
    <span class="mono">versions</span> folder for a while.
  </p>
  {#snippet actions()}
    <button class="btn" onclick={() => (confirmDelete = false)}>Keep it</button>
    <button class="btn btn-danger" onclick={remove} disabled={deleting}>
      Delete {isSeed ? "seed phrase" : "note"}
    </button>
  {/snippet}
</Dialog>

<style>
  .view {
    display: grid;
    gap: var(--s-5);
    max-width: 820px;
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-3);
  }
  .titles {
    display: grid;
    gap: var(--s-1);
    min-width: 0;
  }
  h2 {
    font-size: var(--text-lg);
    overflow-wrap: anywhere;
  }
  h3 {
    font-size: var(--text-sm);
    font-weight: 650;
  }
  .meta {
    font-size: var(--text-sm);
    color: var(--text-3);
  }
  .tools,
  .plate-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
  .danger {
    color: var(--danger);
  }
  .hold {
    display: grid;
    gap: var(--s-2);
    user-select: none;
    touch-action: none;
  }
  .hold.holding :global(.plate) {
    border-color: var(--accent);
    transition: border-color var(--dur-slow) var(--ease-out);
  }
  .hold-hint {
    font-size: var(--text-xs);
    color: var(--text-3);
  }
  .check {
    display: inline-flex;
  }
  .check span {
    display: inline-flex;
    align-items: center;
    gap: var(--s-1);
    font-weight: 600;
  }
  .ok {
    color: var(--ok);
  }
  .caution {
    color: var(--caution);
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    padding: var(--s-4);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    background: var(--surface-raised);
  }
  .secret {
    margin-top: var(--s-1);
    overflow-wrap: anywhere;
  }
  .notes {
    display: grid;
    gap: var(--s-3);
    padding: var(--s-4);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    background: var(--surface-raised);
  }
  .notes-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-2);
  }
  .body {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    max-width: var(--measure);
  }
  .redacted {
    display: grid;
    gap: var(--s-2);
    padding: var(--s-1) 0;
  }
  .redacted span {
    height: 10px;
    border-radius: 5px;
    background: var(--surface-sunk);
  }
</style>
