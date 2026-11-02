<script lang="ts">
  import { api, errorText, MIN_COMBINED_BITS, type Status, type Strength } from "../lib/api";
  import Icon from "../components/Icon.svelte";
  import FolderChoice from "../components/FolderChoice.svelte";
  import PasswordField from "../components/PasswordField.svelte";
  import StrengthMeter from "../components/StrengthMeter.svelte";
  import SeedPlate from "../components/SeedPlate.svelte";

  let { onBack, onCreated }: { onBack: () => void; onCreated: (s: Status) => void } = $props();

  const steps = ["Vault folder", "First password", "Second password", "Confirm both", "Seal the vault"];
  const PER_PASSWORD_TARGET = MIN_COMBINED_BITS / 2;

  let step = $state(0);
  let folder = $state("");
  let first = $state("");
  let second = $state("");
  let confirmFirst = $state("");
  let confirmSecond = $state("");
  let understood = $state(false);
  let strength = $state<Strength | null>(null);
  let error = $state("");
  let confirmErrors = $state({ first: "", second: "" });
  let sealing = $state(false);
  let heading: HTMLHeadingElement | undefined = $state();

  let timer: ReturnType<typeof setTimeout> | undefined;
  function measure() {
    clearTimeout(timer);
    timer = setTimeout(async () => {
      strength = await api.estimateStrength(first, second);
    }, 120);
  }

  async function generate(which: "first" | "second") {
    const phrase = await api.generatePassphrase();
    if (which === "first") first = phrase;
    else second = phrase;
    measure();
  }

  function go(to: number) {
    error = "";
    step = to;
    queueMicrotask(() => heading?.focus());
  }

  async function next(e: SubmitEvent) {
    e.preventDefault();
    error = "";
    if (step === 1 || step === 2) {
      clearTimeout(timer);
      strength = await api.estimateStrength(first, second);
    }
    if (step === 0 && !folder) {
      error = "Choose where the vault should live.";
      return;
    }
    if (step === 1 && (strength?.firstBits ?? 0) < 30) {
      error = "This password is too easy to guess. Make it longer, or use Generate.";
      return;
    }
    if (step === 2) {
      if (strength?.problems.includes("same")) {
        error = "Use a different password from the first one.";
        return;
      }
      if (!strength?.acceptable) {
        error = "Together the two passwords are still too easy to guess. Lengthen this one, or use Generate.";
        return;
      }
    }
    if (step === 3) {
      confirmErrors = {
        first: confirmFirst === first ? "" : "This doesn't match your first password.",
        second: confirmSecond === second ? "" : "This doesn't match your second password.",
      };
      if (confirmErrors.first || confirmErrors.second) return;
    }
    go(step + 1);
  }

  async function seal(e: SubmitEvent) {
    e.preventDefault();
    if (!understood) {
      error = "Tick the box to confirm you understand that lost passwords can't be recovered.";
      return;
    }
    sealing = true;
    error = "";
    try {
      onCreated(await api.createVault(folder, first, second));
    } catch (err) {
      error = errorText(err);
      sealing = false;
    }
  }
</script>

<main class="create">
  <aside class="rail">
    <button class="btn btn-quiet back" onclick={onBack} disabled={sealing}><Icon name="back" /> Back</button>
    <ol class="steps" aria-label="Steps">
      {#each steps as label, i (label)}
        <li class:done={i < step} class:current={i === step} aria-current={i === step ? "step" : undefined}>
          <span class="n mono">{i < step ? "✓" : i + 1}</span>
          {label}
        </li>
      {/each}
    </ol>
    <p class="compact-step mono">Step {step + 1} of {steps.length}</p>
  </aside>

  <section class="panel">
    {#if step === 0}
      <form onsubmit={next} class="stack">
        <h1 bind:this={heading} tabindex="-1">Where should the vault live?</h1>
        <p class="lead">
          Choose a folder inside Google Drive so the sealed vault reaches your Google account. Keyplate writes
          <span class="mono">vault.keyplate</span> there and keeps earlier versions beside it.
        </p>
        <FolderChoice bind:value={folder} purpose="create" />
        {#if error}<p class="error-text" role="alert">{error}</p>{/if}
        <div><button class="btn btn-primary">Continue</button></div>
      </form>
    {:else if step === 1}
      <form onsubmit={next} class="stack">
        <h1 bind:this={heading} tabindex="-1">Choose your first password</h1>
        <p class="lead">
          A passphrase of four or more random words is strong and easy to remember. Generate makes one with 55 bits of
          real randomness.
        </p>
        <PasswordField id="first" label="First password" bind:value={first} oninput={measure} onGenerate={() => generate("first")} autofocus error={error} />
        <StrengthMeter bits={strength?.firstBits ?? 0} target={PER_PASSWORD_TARGET} label="First password strength" />
        <div><button class="btn btn-primary">Continue</button></div>
      </form>
    {:else if step === 2}
      <form onsubmit={next} class="stack">
        <h1 bind:this={heading} tabindex="-1">Choose your second password</h1>
        <p class="lead">
          It must be different from the first. Keyplate combines the two, in this order, into the key that seals your
          vault.
        </p>
        <PasswordField id="second" label="Second password" bind:value={second} oninput={measure} onGenerate={() => generate("second")} autofocus error={error} />
        <StrengthMeter bits={strength?.secondBits ?? 0} target={PER_PASSWORD_TARGET} label="Second password strength" />
        <div class="combined">
          <p class="label">Both together</p>
          <StrengthMeter bits={strength?.combinedBits ?? 0} target={MIN_COMBINED_BITS} label="Combined strength" />
        </div>
        <div class="row">
          <button type="button" class="btn" onclick={() => go(1)}>Previous</button>
          <button class="btn btn-primary">Continue</button>
        </div>
      </form>
    {:else if step === 3}
      <form onsubmit={next} class="stack">
        <h1 bind:this={heading} tabindex="-1">Type both again, in order</h1>
        <p class="lead">
          This is the last check before the vault is sealed. If you generated them, write them down now and keep the
          paper away from this PC.
        </p>
        <PasswordField id="confirm-first" label="First password" bind:value={confirmFirst} autofocus error={confirmErrors.first} />
        <PasswordField id="confirm-second" label="Second password" bind:value={confirmSecond} error={confirmErrors.second} />
        <div class="row">
          <button type="button" class="btn" onclick={() => go(2)}>Previous</button>
          <button class="btn btn-primary">Continue</button>
        </div>
      </form>
    {:else}
      <form onsubmit={seal} class="stack">
        <h1 bind:this={heading} tabindex="-1">Seal the vault</h1>
        {#if sealing}
          <div class="sealing" role="status">
            <SeedPlate count={12} stamping compact label="Sealing" />
            <p>Sealing your vault. Deriving the key takes a second or two on purpose.</p>
          </div>
        {:else}
          <dl class="summary">
            <div><dt>Folder</dt><dd class="mono">{folder}</dd></div>
            <div><dt>Unlock with</dt><dd>First password, then second password</dd></div>
          </dl>
          <StrengthMeter bits={strength?.combinedBits ?? 0} target={MIN_COMBINED_BITS} label="Combined strength" />
          <label class="ack">
            <input type="checkbox" bind:checked={understood} />
            <span>
              I understand that if I forget either password, or their order, nobody can recover this vault. Not
              Google, and not Keyplate.
            </span>
          </label>
          {#if error}<p class="error-text" role="alert">{error}</p>{/if}
          <div class="row">
            <button type="button" class="btn" onclick={() => go(3)}>Previous</button>
            <button class="btn btn-primary">Create vault</button>
          </div>
        {/if}
      </form>
    {/if}
  </section>
</main>

<style>
  .create {
    display: grid;
    grid-template-columns: 240px minmax(0, 1fr);
    gap: var(--s-7);
    max-width: 1040px;
    margin: 0 auto;
    padding: var(--s-6) var(--s-6) var(--s-7);
  }
  .rail {
    display: grid;
    align-content: start;
    gap: var(--s-5);
  }
  .back {
    justify-self: start;
    margin-left: calc(-1 * var(--s-3));
  }
  .steps {
    display: grid;
    gap: var(--s-1);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .steps li {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-2) 0;
    font-size: var(--text-sm);
    color: var(--text-3);
  }
  .steps .n {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    background: var(--surface-raised);
  }
  .steps .current {
    color: var(--text);
    font-weight: 650;
  }
  .steps .current .n {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--on-accent);
  }
  .steps .done {
    color: var(--text-2);
  }
  .steps .done .n {
    border-color: var(--ok);
    color: var(--ok);
  }
  .compact-step {
    display: none;
    font-size: var(--text-xs);
    color: var(--text-3);
  }

  .panel {
    max-width: 620px;
    padding-top: var(--s-7);
  }
  .stack {
    display: grid;
    gap: var(--s-5);
  }
  h1 {
    font-size: var(--text-xl);
  }
  h1:focus {
    outline: none;
  }
  .lead {
    color: var(--text-2);
    max-width: var(--measure);
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-3);
  }
  .combined {
    display: grid;
    gap: var(--s-2);
    padding-top: var(--s-4);
    border-top: 1px solid var(--line);
  }
  .summary {
    display: grid;
    gap: var(--s-3);
    margin: 0;
  }
  .summary div {
    display: grid;
    grid-template-columns: 120px minmax(0, 1fr);
    gap: var(--s-3);
  }
  dt {
    font-size: var(--text-sm);
    color: var(--text-3);
  }
  dd {
    margin: 0;
    overflow-wrap: anywhere;
  }
  .ack {
    display: flex;
    gap: var(--s-3);
    align-items: flex-start;
    padding: var(--s-4);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius);
    background: var(--surface-raised);
    cursor: pointer;
  }
  .ack input {
    flex: none;
    width: 18px;
    height: 18px;
    margin-top: 2px;
    accent-color: var(--accent);
  }
  .sealing {
    display: grid;
    gap: var(--s-4);
    color: var(--text-2);
  }

  @media (max-width: 760px) {
    .create {
      grid-template-columns: minmax(0, 1fr);
      gap: var(--s-4);
      padding: var(--s-4) var(--gutter) var(--s-7);
    }
    .rail {
      grid-template-columns: auto 1fr;
      align-items: center;
    }
    .steps {
      display: none;
    }
    .compact-step {
      display: block;
      justify-self: end;
    }
    .panel {
      padding-top: 0;
    }
    .summary div {
      grid-template-columns: minmax(0, 1fr);
      gap: 0;
    }
  }
</style>
