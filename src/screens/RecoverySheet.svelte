<!-- A printable page that tells future-you how to restore the vault. It holds no secrets. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorText, type RecoveryInfo } from "../lib/api";
  import Icon from "../components/Icon.svelte";

  let info = $state<RecoveryInfo | null>(null);
  let error = $state("");
  const printed = new Date().toLocaleDateString(undefined, { day: "numeric", month: "long", year: "numeric" });

  onMount(async () => {
    try {
      info = await api.recoveryInfo();
    } catch (e) {
      error = errorText(e);
    }
  });
</script>

<div class="screen-tools">
  <div>
    <h2>Recovery sheet</h2>
    <p class="hint">Print this and keep it somewhere safe, apart from anything with your passwords on it.</p>
  </div>
  <button class="btn btn-primary" onclick={() => window.print()} disabled={!info}><Icon name="print" /> Print</button>
</div>

{#if error}<p class="notice notice-danger" role="alert">{error}</p>{/if}

{#if info}
  <article class="sheet">
    <header>
      <p class="mark"><Icon name="plate" size={20} /> Keyplate</p>
      <h1>Recovery sheet</h1>
      <p>This sheet holds no passwords and no seed words. On its own it opens nothing.</p>
    </header>

    <section>
      <h3>Where the vault lives</h3>
      <p class="mono path">{info.folder}</p>
      <p class="line">Google account signed in to Google Drive: <span class="blank"></span></p>
    </section>

    <section>
      <h3>To restore on a new PC</h3>
      <ol>
        <li>Install <strong>Google Drive for desktop</strong> and sign in with the Google account above.</li>
        <li>Wait for <span class="mono">My Drive\Keyplate</span> to sync. Right-click it and choose <em>Available offline</em>.</li>
        <li>
          Install Keyplate from <span class="mono">My Drive\Keyplate\installer</span>. Before running it, check its
          SHA-256 matches the one below.
        </li>
        <li>Open Keyplate, choose <strong>Open an existing vault</strong>, then pick the Keyplate folder.</li>
        <li>Type the <strong>first password</strong>, then the <strong>second password</strong>. Order matters.</li>
      </ol>
    </section>

    <section>
      <h3>Installer check</h3>
      {#if info.installerSha256}
        <p class="mono hash">{info.installerSha256}</p>
      {:else}
        <p>SHA-256 of the installer: <span class="blank wide"></span></p>
      {/if}
      <p class="small">
        In PowerShell: <span class="mono">Get-FileHash .\Keyplate_{info.appVersion}_x64-setup.exe</span>
      </p>
    </section>

    <section>
      <h3>If something goes wrong</h3>
      <ul>
        <li>The <span class="mono">versions</span> folder beside the vault holds the last 30 saves.</li>
        <li>Google Drive keeps its own version history: right-click <span class="mono">vault.keyplate</span>, then File information, then Manage versions.</li>
        <li>Nobody can reset the passwords. Without both, in order, the vault cannot be opened.</li>
      </ul>
    </section>

    <footer class="small mono">
      Keyplate {info.appVersion} · XChaCha20-Poly1305, 256-bit key · {info.kdf} · Printed {printed}
    </footer>
  </article>
{/if}

<style>
  .screen-tools {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-3);
    max-width: 760px;
    margin-bottom: var(--s-5);
  }
  .screen-tools h2 {
    font-size: var(--text-lg);
  }
  .sheet {
    display: grid;
    gap: var(--s-5);
    max-width: 760px;
    padding: var(--s-6);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    background: var(--surface-raised);
  }
  header {
    display: grid;
    gap: var(--s-2);
    padding-bottom: var(--s-4);
    border-bottom: 2px solid var(--text);
  }
  .mark {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    font-weight: 700;
  }
  h1 {
    font-size: var(--text-xl);
  }
  h3 {
    margin-bottom: var(--s-2);
    font-size: var(--text-base);
  }
  ol,
  ul {
    display: grid;
    gap: var(--s-2);
    margin: 0;
    padding-left: var(--s-5);
  }
  .path,
  .hash {
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    overflow-wrap: anywhere;
  }
  .line {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    align-items: flex-end;
    margin-top: var(--s-3);
  }
  .blank {
    display: inline-block;
    flex: 1;
    min-width: 200px;
    border-bottom: 1px solid var(--text-3);
    height: 1.2em;
  }
  .small {
    margin-top: var(--s-2);
    font-size: var(--text-sm);
    color: var(--text-2);
  }
  footer {
    padding-top: var(--s-3);
    border-top: 1px solid var(--line);
    overflow-wrap: anywhere;
  }

  @media print {
    .screen-tools {
      display: none;
    }
    .sheet {
      max-width: none;
      padding: 0;
      border: 0;
      background: #fff;
      color: #000;
    }
    .small {
      color: #333;
    }
  }
</style>
