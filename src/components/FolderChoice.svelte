<!-- Picks the vault folder: detected Google Drive locations first, any folder as a fallback. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/api";
  import Icon from "./Icon.svelte";

  let { value = $bindable(""), purpose }: { value?: string; purpose: "create" | "open" } = $props();

  let suggestions = $state<string[]>([]);
  let loaded = $state(false);
  let custom = $state<string | null>(null);

  onMount(async () => {
    suggestions = await api.driveSuggestions();
    loaded = true;
    if (!value && suggestions.length) value = suggestions[0];
  });

  async function pick() {
    const folder = await api.pickFolder();
    if (folder) {
      custom = folder;
      value = folder;
    }
  }

  const options = $derived([...suggestions, ...(custom && !suggestions.includes(custom) ? [custom] : [])]);
</script>

<fieldset class="choice">
  <legend class="label">Vault folder</legend>

  {#if loaded && suggestions.length === 0}
    <div class="notice notice-caution">
      <Icon name="alert" />
      <p>
        <strong>Google Drive for desktop isn't running on this PC.</strong>
        {#if purpose === "create"}
          Install it and sign in, so the vault reaches your Google account and you can restore it on another PC. You can
          also choose any folder now and move the vault later.
        {:else}
          Install it, sign in, and wait for <span class="mono">My Drive\Keyplate</span> to appear. Or choose a folder that
          holds a copy of <span class="mono">vault.keyplate</span>.
        {/if}
      </p>
    </div>
  {/if}

  {#each options as folder (folder)}
    <label class="option">
      <input type="radio" name="vault-folder-{purpose}" value={folder} bind:group={value} />
      <span class="path mono">{folder}</span>
      {#if suggestions.includes(folder)}<span class="tag">Google Drive</span>{/if}
    </label>
  {/each}

  <div>
    <button type="button" class="btn" onclick={pick}><Icon name="folder" /> Choose another folder</button>
  </div>
</fieldset>

<style>
  .choice {
    display: grid;
    gap: var(--s-2);
    margin: 0;
    padding: 0;
    border: 0;
    min-width: 0;
  }
  legend {
    margin-bottom: var(--s-2);
    padding: 0;
  }
  .option {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    min-height: 48px;
    padding: var(--s-2) var(--s-4);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius);
    background: var(--surface-raised);
    cursor: pointer;
    min-width: 0;
  }
  .option:has(input:checked) {
    border-color: var(--accent);
    background: var(--accent-tint);
  }
  .option input {
    flex: none;
    accent-color: var(--accent);
    width: 16px;
    height: 16px;
  }
  .path {
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
    font-size: var(--text-sm);
  }
  .tag {
    flex: none;
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-2);
  }
</style>
