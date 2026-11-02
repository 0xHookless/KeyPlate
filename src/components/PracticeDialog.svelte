<script lang="ts">
  import { api, errorText } from "../lib/api";
  import Dialog from "./Dialog.svelte";
  import PasswordField from "./PasswordField.svelte";

  let { open = $bindable(false), onDone }: { open?: boolean; onDone?: () => void } = $props();

  let first = $state("");
  let second = $state("");
  let busy = $state(false);
  let result = $state<"" | "ok" | "no">("");
  let error = $state("");

  $effect(() => {
    if (open) {
      first = second = "";
      result = "";
      error = "";
    }
  });

  async function practise(e: SubmitEvent) {
    e.preventDefault();
    busy = true;
    error = "";
    try {
      result = (await api.practiceUnlock(first, second)) ? "ok" : "no";
      if (result === "ok") onDone?.();
    } catch (err) {
      error = errorText(err);
    } finally {
      busy = false;
      first = second = "";
    }
  }
</script>

<Dialog bind:open title="Practise unlocking">
  <form id="practice-form" class="stack" onsubmit={practise}>
    <p>Type both passwords, in order. Nothing changes; this only checks you still remember them.</p>
    <PasswordField id="practice-first" label="First password" bind:value={first} autofocus disabled={busy} />
    <PasswordField id="practice-second" label="Second password" bind:value={second} disabled={busy} />
    {#if result === "ok"}
      <p class="notice notice-ok" role="status">Both correct, in the right order. See you next time.</p>
    {:else if result === "no"}
      <p class="notice notice-caution" role="alert">
        Those don't match. Try again, and check the order. While the vault is unlocked you can also set new passwords
        in Settings.
      </p>
    {/if}
    {#if error}<p class="error-text" role="alert">{error}</p>{/if}
  </form>
  {#snippet actions()}
    <button class="btn" onclick={() => (open = false)}>Close</button>
    <button class="btn btn-primary" form="practice-form" disabled={busy}>{busy ? "Checking…" : "Check passwords"}</button>
  {/snippet}
</Dialog>

<style>
  .stack {
    display: grid;
    gap: var(--s-4);
  }
</style>
