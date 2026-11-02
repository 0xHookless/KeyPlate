<script lang="ts">
  import { api, errorText, type Status } from "../lib/api";
  import Icon from "../components/Icon.svelte";
  import SeedPlate from "../components/SeedPlate.svelte";
  import FolderChoice from "../components/FolderChoice.svelte";
  import CreateVault from "./CreateVault.svelte";

  let { onStatus }: { onStatus: (s: Status, created: boolean) => void } = $props();

  let view = $state<"intro" | "create" | "open">("intro");
  let folder = $state("");
  let opening = $state(false);
  let error = $state("");

  async function openExisting(e: SubmitEvent) {
    e.preventDefault();
    if (!folder) {
      error = "Choose the folder that holds vault.keyplate.";
      return;
    }
    opening = true;
    error = "";
    try {
      onStatus(await api.useExistingVault(folder), false);
    } catch (err) {
      error = errorText(err);
    } finally {
      opening = false;
    }
  }
</script>

{#if view === "create"}
  <CreateVault onBack={() => (view = "intro")} onCreated={(s) => onStatus(s, true)} />
{:else if view === "open"}
  <main class="narrow">
    <button class="btn btn-quiet back" onclick={() => (view = "intro")}><Icon name="back" /> Back</button>
    <h1>Open an existing vault</h1>
    <p class="lead">
      Choose the folder that holds <span class="mono">vault.keyplate</span>. It's usually
      <span class="mono">My Drive\Keyplate</span>. Next you'll type both passwords.
    </p>
    <form onsubmit={openExisting} class="stack">
      <FolderChoice bind:value={folder} purpose="open" />
      {#if error}<p class="notice notice-danger" role="alert">{error}</p>{/if}
      <div><button class="btn btn-primary" disabled={opening}>Open vault</button></div>
    </form>
  </main>
{:else}
  <main class="intro">
    <section class="copy">
      <p class="mark"><Icon name="plate" size={22} /> Keyplate</p>
      <h1>Seed phrases, sealed before they reach the cloud.</h1>
      <p class="lead">
        Keyplate encrypts seed phrases and private notes on this PC, then lets Google Drive carry the sealed file to
        your other computers.
      </p>
      <div class="actions">
        <button class="btn btn-primary" onclick={() => (view = "create")}>Create a new vault</button>
        <button class="btn" onclick={() => (view = "open")}>Open an existing vault</button>
      </div>

      <dl class="facts">
        <div>
          <dt>Two passwords, in order</dt>
          <dd>Both are needed, first then second. Nobody else can reset or recover them, so nobody else can get in.</dd>
        </div>
        <div>
          <dt>Encrypted before it syncs</dt>
          <dd>Google Drive only ever stores a sealed file. Without your passwords it is random bytes.</dd>
        </div>
        <div>
          <dt>Comes back on any PC</dt>
          <dd>Install Google Drive and Keyplate, choose the folder, type both passwords.</dd>
        </div>
        <div>
          <dt>Quantum-resistant by design</dt>
          <dd>256-bit symmetric encryption, with no public-key cryptography for a quantum computer to break.</dd>
        </div>
      </dl>
    </section>

    <section class="art" aria-hidden="true">
      <SeedPlate count={12} label="Empty plate" />
      <p class="caption mono">XChaCha20-Poly1305 · Argon2id · 256-bit</p>
    </section>
  </main>
{/if}

<style>
  .intro {
    display: grid;
    grid-template-columns: minmax(0, 1.05fr) minmax(0, 1fr);
    gap: var(--s-8);
    align-items: center;
    max-width: 1120px;
    min-height: 100vh;
    margin: 0 auto;
    padding: var(--s-7) var(--s-7);
  }
  .copy {
    display: grid;
    gap: var(--s-5);
  }
  .mark {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    font-weight: 700;
    font-size: var(--text-md);
    color: var(--accent);
  }
  h1 {
    font-size: var(--text-2xl);
    font-weight: 700;
    letter-spacing: -0.02em;
    max-width: 16ch;
  }
  .lead {
    max-width: 52ch;
    font-size: var(--text-md);
    color: var(--text-2);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-3);
  }
  .facts {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--s-4) var(--s-6);
    margin: var(--s-4) 0 0;
    padding-top: var(--s-5);
    border-top: 1px solid var(--line);
  }
  dt {
    font-weight: 650;
    font-size: var(--text-sm);
  }
  dd {
    margin: var(--s-1) 0 0;
    font-size: var(--text-sm);
    color: var(--text-2);
  }
  .art {
    display: grid;
    gap: var(--s-3);
  }
  .caption {
    justify-self: end;
    font-size: var(--text-xs);
    color: var(--text-3);
  }

  .narrow {
    display: grid;
    gap: var(--s-4);
    max-width: 640px;
    margin: 0 auto;
    padding: var(--s-6) var(--gutter) var(--s-7);
  }
  .narrow h1 {
    font-size: var(--text-xl);
    max-width: none;
  }
  .back {
    justify-self: start;
    margin-left: calc(-1 * var(--s-3));
  }
  .stack {
    display: grid;
    gap: var(--s-5);
    margin-top: var(--s-2);
  }

  @media (max-width: 880px) {
    .intro {
      grid-template-columns: minmax(0, 1fr);
      gap: var(--s-6);
      min-height: 0;
      padding: var(--s-6) var(--gutter) var(--s-7);
    }
    h1 {
      font-size: var(--text-xl);
    }
  }
  @media (max-width: 520px) {
    .facts {
      grid-template-columns: minmax(0, 1fr);
    }
    .actions .btn {
      width: 100%;
    }
  }
</style>
