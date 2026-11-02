<script lang="ts">
  import { api, errorText, type Settings, type Status } from "../lib/api";
  import { applyTheme, getTheme, type Theme } from "../lib/theme";
  import { notify } from "../lib/toast.svelte";
  import Icon from "../components/Icon.svelte";
  import PracticeDialog from "../components/PracticeDialog.svelte";
  import ChangePasswordsDialog from "../components/ChangePasswordsDialog.svelte";

  let {
    status,
    onStatus,
    onRecovery,
  }: { status: Status; onStatus: (s: Status) => void; onRecovery: () => void } = $props();

  let theme = $state<Theme>(getTheme());
  let practiceOpen = $state(false);
  let changeOpen = $state(false);
  let exporting = $state(false);

  async function update(patch: Partial<Settings>) {
    try {
      onStatus(await api.saveSettings({ ...status.settings, ...patch }));
    } catch (e) {
      notify(errorText(e), "danger");
    }
  }

  async function exportBackup() {
    exporting = true;
    try {
      const path = await api.exportBackup();
      if (path) notify(`Saved an encrypted backup to ${path}. It opens with the same two passwords.`, "ok", 7000);
    } catch (e) {
      notify(errorText(e), "danger");
    } finally {
      exporting = false;
    }
  }

  async function forget() {
    onStatus(await api.forgetVault());
  }
</script>

<div class="settings">
  <h2>Settings</h2>

  <section>
    <h3>Locking</h3>
    <div class="field">
      <label for="auto-lock">Lock after being idle for</label>
      <select id="auto-lock" class="select" value={status.settings.autoLockMinutes} onchange={(e) => update({ autoLockMinutes: +e.currentTarget.value })}>
        {#each [1, 2, 5, 10, 15, 30, 60] as m (m)}<option value={m}>{m} {m === 1 ? "minute" : "minutes"}</option>{/each}
      </select>
      <p class="hint">Keyplate also locks when Windows locks or goes to sleep.</p>
    </div>
    <div class="field">
      <label for="clipboard">Clear copied words after</label>
      <select id="clipboard" class="select" value={status.settings.clipboardSeconds} onchange={(e) => update({ clipboardSeconds: +e.currentTarget.value })}>
        {#each [10, 20, 30, 60, 120] as s (s)}<option value={s}>{s} seconds</option>{/each}
      </select>
      <p class="hint">Copies never enter Windows clipboard history or the cloud clipboard.</p>
    </div>
  </section>

  <section>
    <h3>Passwords</h3>
    <div class="field">
      <label for="practice">Remind me to practise unlocking every</label>
      <select id="practice" class="select" value={status.settings.practiceDays} onchange={(e) => update({ practiceDays: +e.currentTarget.value })}>
        <option value={0}>Never</option>
        {#each [14, 30, 60, 90] as d (d)}<option value={d}>{d} days</option>{/each}
      </select>
      <p class="hint">Typing them now and then is the best protection against forgetting.</p>
    </div>
    <div class="buttons">
      <button class="btn" onclick={() => (practiceOpen = true)}><Icon name="key" /> Practise now</button>
      <button class="btn" onclick={() => (changeOpen = true)}>Change passwords</button>
    </div>
  </section>

  <section>
    <h3>Backup and recovery</h3>
    <dl class="facts">
      <div><dt>Vault folder</dt><dd class="mono">{status.folder}</dd></div>
    </dl>
    <p class="hint">
      Google Drive syncs this folder. Keyplate also keeps the last 30 saves in its <span class="mono">versions</span>
      folder.
    </p>
    <div class="buttons">
      <button class="btn" onclick={onRecovery}><Icon name="print" /> Recovery sheet</button>
      <button class="btn" onclick={exportBackup} disabled={exporting}><Icon name="download" /> Export encrypted backup</button>
    </div>
  </section>

  <section>
    <h3>Appearance</h3>
    <fieldset class="themes">
      <legend class="visually-hidden">Theme</legend>
      {#each [["system", "Match Windows"], ["light", "Light"], ["dark", "Dark"]] as [value, label] (value)}
        <label class="segment">
          <input type="radio" name="theme" {value} bind:group={theme} onchange={() => applyTheme(theme)} />
          {label}
        </label>
      {/each}
    </fieldset>
  </section>

  <section>
    <h3>This vault</h3>
    <p class="hint">
      Opening a different vault locks this one and forgets its folder on this PC. Nothing is deleted.
    </p>
    <div><button class="btn" onclick={forget}>Open a different vault</button></div>
  </section>
</div>

<PracticeDialog bind:open={practiceOpen} />
<ChangePasswordsDialog bind:open={changeOpen} />

<style>
  .settings {
    display: grid;
    gap: var(--s-6);
    max-width: 640px;
  }
  h2 {
    font-size: var(--text-lg);
  }
  section {
    display: grid;
    gap: var(--s-4);
    padding-top: var(--s-5);
    border-top: 1px solid var(--line);
  }
  h3 {
    font-size: var(--text-md);
  }
  .select {
    max-width: 280px;
  }
  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
  .facts {
    margin: 0;
  }
  dt {
    font-size: var(--text-sm);
    color: var(--text-3);
  }
  dd {
    margin: var(--s-1) 0 0;
    overflow-wrap: anywhere;
  }
  .themes {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin: 0;
    padding: 0;
    border: 0;
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
    margin: 0;
    accent-color: var(--accent);
  }
</style>
