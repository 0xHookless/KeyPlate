<script lang="ts">
  import { untrack } from "svelte";
  import { api, errorText, type EntryDetail, type EntryKind, type EntrySummary, type SeedCheck } from "../lib/api";
  import Icon from "../components/Icon.svelte";
  import SeedInput from "../components/SeedInput.svelte";
  import PasswordField from "../components/PasswordField.svelte";

  let {
    initial,
    kind,
    onSaved,
    onCancel,
  }: {
    initial: EntryDetail | null;
    kind: EntryKind;
    onSaved: (s: EntrySummary) => void;
    onCancel: () => void;
  } = $props();

  const STANDARD = [12, 15, 18, 21, 24];
  const NETWORKS = ["Bitcoin", "Ethereum", "Solana", "Cardano", "Litecoin", "Monero", "Polkadot", "XRP"];

  // Snapshot of the entry being edited; later prop changes must not overwrite typing.
  const start = untrack(() => $state.snapshot(initial));
  const startCount = start?.words.length || 12;

  let title = $state(start?.title ?? "");
  let network = $state(start?.network ?? "");
  let words = $state<string[]>(start?.words.length ? [...start.words] : Array(startCount).fill(""));
  let passphrase = $state(start?.bip39Passphrase ?? "");
  let body = $state(start?.body ?? "");
  let customCount = $state(STANDARD.includes(startCount) ? 25 : startCount);
  let check = $state<SeedCheck | null>(null);
  let saving = $state(false);
  let error = $state("");

  const isSeed = untrack(() => kind === "seed");
  const count = $derived(words.length);
  const standard = $derived(STANDARD.includes(count));
  const invalid = $derived(new Set(check?.status === "unknownWords" ? check.words.map((w) => w.index) : []));

  function resize(n: number) {
    words = Array.from({ length: n }, (_, i) => words[i] ?? "");
    checkSoon();
  }

  let timer: ReturnType<typeof setTimeout> | undefined;
  function checkSoon() {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      check = words.some((w) => w.trim()) ? await api.checkSeed(words) : null;
    }, 200);
  }
  if (isSeed && start?.words.length) checkSoon();

  function useSuggestion(index: number, word: string) {
    words[index] = word;
    checkSoon();
  }

  async function save(e: SubmitEvent) {
    e.preventDefault();
    error = "";
    if (isSeed && !words.some((w) => w.trim())) {
      error = "Type the words of the seed phrase, or save this as a note instead.";
      return;
    }
    if (!isSeed && !body.trim()) {
      error = "Write the text you want to keep.";
      return;
    }
    saving = true;
    try {
      onSaved(
        await api.saveEntry({
          id: start?.id ?? null,
          kind,
          title,
          words: isSeed ? words : [],
          bip39Passphrase: isSeed && passphrase ? passphrase : null,
          network: isSeed && network ? network : null,
          body,
        }),
      );
    } catch (err) {
      error = errorText(err);
      saving = false;
    }
  }
</script>

<form class="editor" onsubmit={save}>
  <header>
    <h2>{start ? `Edit ${isSeed ? "seed phrase" : "note"}` : isSeed ? "New seed phrase" : "New note"}</h2>
    <p class="hint">Saved encrypted. Google Drive only ever receives the sealed file.</p>
  </header>

  <div class="field">
    <label for="entry-title">Name</label>
    <input id="entry-title" class="input" bind:value={title} placeholder={isSeed ? "e.g. Hardware wallet" : "e.g. Exchange recovery codes"} />
    <p class="hint">Only you see this. Names stay readable in the list; everything else stays hidden.</p>
  </div>

  {#if isSeed}
    <div class="field">
      <label for="entry-network">Coin or wallet <span class="optional">(optional)</span></label>
      <input id="entry-network" class="input" list="networks" bind:value={network} />
      <datalist id="networks">{#each NETWORKS as n (n)}<option value={n}></option>{/each}</datalist>
    </div>

    <fieldset class="count">
      <legend class="label">Number of words</legend>
      <div class="segments">
        {#each STANDARD as n (n)}
          <label class="segment"><input type="radio" name="count" checked={count === n} onchange={() => resize(n)} /> {n}</label>
        {/each}
        <label class="segment"><input type="radio" name="count" checked={!standard} onchange={() => resize(customCount)} /> Other</label>
        {#if !standard}
          <label class="custom">
            <span class="visually-hidden">Custom word count</span>
            <input
              class="input"
              type="number"
              min="1"
              max="48"
              bind:value={customCount}
              onchange={() => resize(Math.max(1, Math.min(48, customCount || 1)))}
            />
          </label>
        {/if}
      </div>
    </fieldset>

    <div class="field">
      <span class="label" id="words-label">Words, in order</span>
      <p class="hint">Paste the whole phrase into word 01, or type each word. Space moves to the next word.</p>
      <SeedInput bind:words {invalid} onchange={checkSoon} />
      <div class="status" aria-live="polite">
        {#if check?.status === "valid"}
          <p class="state ok"><Icon name="check" /> Checksum valid. These words form a real BIP39 seed phrase.</p>
        {:else if check?.status === "badChecksum"}
          <p class="state caution">
            <Icon name="alert" /> Every word is real, but the checksum doesn't match. A word is probably mistyped or out
            of order. You can still save it.
          </p>
        {:else if check?.status === "unknownWords"}
          <div class="state danger">
            <Icon name="alert" />
            <div>
              {#each check.words as w (w.index)}
                <p>
                  Word {w.index + 1}, “{words[w.index]}”, isn't a BIP39 word.
                  {#if w.suggestions.length}
                    Did you mean
                    {#each w.suggestions as s, k (s)}
                      <button type="button" class="link-btn" onclick={() => useSuggestion(w.index, s)}>{s}</button>{k < w.suggestions.length - 1 ? ", " : "?"}
                    {/each}
                  {/if}
                </p>
              {/each}
            </div>
          </div>
        {:else if check?.status === "incomplete" && standard}
          <p class="state">{check.filled} of {count} words entered.</p>
        {:else if check && !standard}
          <p class="state">Not a standard BIP39 length, so Keyplate can't check it. That's fine for wallets with their own word lists.</p>
        {/if}
      </div>
    </div>

    <PasswordField
      id="entry-passphrase"
      label="BIP39 passphrase (optional)"
      bind:value={passphrase}
      hint="Only if your wallet uses an extra passphrase, sometimes called the 25th word."
    />
  {/if}

  <div class="field">
    <label for="entry-body">{isSeed ? "Notes (optional)" : "Text"}</label>
    <textarea id="entry-body" class="textarea" class:tall={!isSeed} bind:value={body} spellcheck="false"></textarea>
    {#if isSeed}<p class="hint">Where the hardware wallet is, derivation paths, anything future-you needs.</p>{/if}
  </div>

  {#if error}<p class="notice notice-danger" role="alert">{error}</p>{/if}

  <div class="actions">
    <button class="btn btn-primary" disabled={saving}>{isSeed ? "Save seed phrase" : "Save note"}</button>
    <button type="button" class="btn" onclick={onCancel} disabled={saving}>Cancel</button>
  </div>
</form>

<style>
  .editor {
    display: grid;
    gap: var(--s-5);
    max-width: 760px;
  }
  header {
    display: grid;
    gap: var(--s-1);
  }
  h2 {
    font-size: var(--text-lg);
  }
  .optional {
    font-weight: 400;
    color: var(--text-3);
  }
  .count {
    margin: 0;
    padding: 0;
    border: 0;
    min-width: 0;
  }
  .count legend {
    margin-bottom: var(--s-2);
    padding: 0;
  }
  .segments {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    align-items: center;
  }
  .segment {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    min-height: 36px;
    padding: 0 var(--s-3);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius);
    background: var(--surface-raised);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    cursor: pointer;
  }
  .segment:has(input:checked) {
    border-color: var(--accent);
    background: var(--accent-tint);
  }
  .segment:has(input:focus-visible) {
    box-shadow: var(--focus-ring);
  }
  .segment input {
    accent-color: var(--accent);
    margin: 0;
  }
  .custom .input {
    width: 88px;
    min-height: 36px;
  }
  .status {
    min-height: 24px;
  }
  .state {
    display: flex;
    gap: var(--s-2);
    align-items: flex-start;
    font-size: var(--text-sm);
    color: var(--text-2);
  }
  .state :global(svg) {
    flex: none;
    margin-top: 2px;
  }
  .state.ok {
    color: var(--ok);
  }
  .state.caution {
    color: var(--caution);
  }
  .state.danger {
    color: var(--danger);
  }
  .state.danger .link-btn {
    color: var(--danger);
  }
  .tall {
    min-height: 220px;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-3);
  }
</style>
