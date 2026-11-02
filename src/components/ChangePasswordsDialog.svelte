<script lang="ts">
  import { api, errorText, MIN_COMBINED_BITS, type Strength } from "../lib/api";
  import { notify } from "../lib/toast.svelte";
  import Dialog from "./Dialog.svelte";
  import PasswordField from "./PasswordField.svelte";
  import StrengthMeter from "./StrengthMeter.svelte";

  let { open = $bindable(false) }: { open?: boolean } = $props();

  let currentFirst = $state("");
  let currentSecond = $state("");
  let newFirst = $state("");
  let newSecond = $state("");
  let strength = $state<Strength | null>(null);
  let busy = $state(false);
  let error = $state("");

  $effect(() => {
    if (open) {
      currentFirst = currentSecond = newFirst = newSecond = "";
      strength = null;
      error = "";
    }
  });

  let timer: ReturnType<typeof setTimeout> | undefined;
  function measure() {
    clearTimeout(timer);
    timer = setTimeout(async () => (strength = await api.estimateStrength(newFirst, newSecond)), 120);
  }

  async function generate(which: "first" | "second") {
    const p = await api.generatePassphrase();
    if (which === "first") newFirst = p;
    else newSecond = p;
    measure();
  }

  async function change(e: SubmitEvent) {
    e.preventDefault();
    busy = true;
    error = "";
    try {
      await api.changePasswords(currentFirst, currentSecond, newFirst, newSecond);
      open = false;
      notify("Passwords changed. Your other PCs pick up the change the next time Drive syncs.", "ok", 6000);
    } catch (err) {
      error = errorText(err);
    } finally {
      busy = false;
    }
  }
</script>

<Dialog bind:open title="Change passwords" width={520}>
  <form id="change-form" class="stack" onsubmit={change}>
    <PasswordField id="cur-first" label="Current first password" bind:value={currentFirst} autofocus disabled={busy} />
    <PasswordField id="cur-second" label="Current second password" bind:value={currentSecond} disabled={busy} />
    <hr />
    <PasswordField id="new-first" label="New first password" bind:value={newFirst} oninput={measure} onGenerate={() => generate("first")} disabled={busy} />
    <PasswordField id="new-second" label="New second password" bind:value={newSecond} oninput={measure} onGenerate={() => generate("second")} disabled={busy} />
    <StrengthMeter bits={strength?.combinedBits ?? 0} target={MIN_COMBINED_BITS} label="New passwords combined strength" />
    <p class="hint">
      Older copies in the <span class="mono">versions</span> folder stay sealed with the old passwords.
    </p>
    {#if error}<p class="notice notice-danger" role="alert">{error}</p>{/if}
  </form>
  {#snippet actions()}
    <button class="btn" onclick={() => (open = false)} disabled={busy}>Cancel</button>
    <button class="btn btn-primary" form="change-form" disabled={busy}>{busy ? "Re-sealing…" : "Change passwords"}</button>
  {/snippet}
</Dialog>

<style>
  .stack {
    display: grid;
    gap: var(--s-4);
  }
  hr {
    width: 100%;
    margin: 0;
    border: 0;
    border-top: 1px solid var(--line);
  }
</style>
