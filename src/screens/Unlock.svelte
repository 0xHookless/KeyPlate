<script lang="ts">
  import { api, errorCode, errorText, type Status } from "../lib/api";
  import PasswordField from "../components/PasswordField.svelte";
  import SeedPlate from "../components/SeedPlate.svelte";

  let { status, onStatus }: { status: Status; onStatus: (s: Status) => void } = $props();

  let first = $state("");
  let second = $state("");
  let busy = $state(false);
  let error = $state("");
  let capsLock = $state(false);

  async function unlock(e: SubmitEvent) {
    e.preventDefault();
    if (!first || !second) {
      error = !first ? "Type your first password." : "Type your second password.";
      return;
    }
    busy = true;
    error = "";
    try {
      const s = await api.unlock(first, second);
      first = second = "";
      onStatus(s);
    } catch (err) {
      error = errorText(err);
      if (errorCode(err) === "wrongPasswords") second = "";
    } finally {
      busy = false;
    }
  }

  async function forget() {
    onStatus(await api.forgetVault());
  }

  function keys(e: KeyboardEvent) {
    capsLock = e.getModifierState?.("CapsLock") ?? false;
  }
</script>

<svelte:window onkeydown={keys} onkeyup={keys} />

<main class="unlock">
  <div class="plate" aria-hidden="true">
    <SeedPlate count={12} stamping={busy} compact label="Vault" />
  </div>

  <form onsubmit={unlock} class="form" aria-busy={busy}>
    <h1>Unlock Keyplate</h1>
    {#if status.folder}<p class="folder mono" title={status.folder}>{status.folder}</p>{/if}

    <PasswordField id="unlock-first" label="First password" bind:value={first} autofocus disabled={busy} />
    <PasswordField id="unlock-second" label="Second password" bind:value={second} disabled={busy} />

    {#if capsLock}<p class="hint" role="status">Caps Lock is on.</p>{/if}
    {#if error}<p class="notice notice-danger" role="alert">{error}</p>{/if}

    <button class="btn btn-primary btn-block" disabled={busy}>{busy ? "Unlocking…" : "Unlock vault"}</button>

    <details class="help">
      <summary>Can't get in?</summary>
      <p>
        Check Caps Lock and the keyboard layout, and that the first password is in the first box. Keyplate can't reset
        passwords. That is what keeps the vault private, even from Google.
      </p>
    </details>
    <button type="button" class="link-btn other" onclick={forget} disabled={busy}>Open a different vault</button>
  </form>
</main>

<style>
  .unlock {
    display: grid;
    align-content: center;
    justify-items: center;
    gap: var(--s-6);
    min-height: 100vh;
    padding: var(--s-6) var(--gutter);
  }
  .plate {
    width: min(440px, 100%);
  }
  .form {
    display: grid;
    gap: var(--s-4);
    width: min(440px, 100%);
  }
  h1 {
    font-size: var(--text-xl);
  }
  .folder {
    margin-top: calc(-1 * var(--s-2));
    font-size: var(--text-xs);
    color: var(--text-3);
    overflow-wrap: anywhere;
  }
  .help {
    font-size: var(--text-sm);
    color: var(--text-2);
  }
  .help summary {
    cursor: pointer;
    font-weight: 600;
    color: var(--text);
  }
  .help p {
    margin-top: var(--s-2);
  }
  .other {
    justify-self: start;
    font-size: var(--text-sm);
  }
</style>
