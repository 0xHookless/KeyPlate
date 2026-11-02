<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    id,
    label,
    value = $bindable(""),
    hint = "",
    error = "",
    autofocus = false,
    disabled = false,
    onGenerate,
    oninput,
  }: {
    id: string;
    label: string;
    value?: string;
    hint?: string;
    error?: string;
    autofocus?: boolean;
    disabled?: boolean;
    onGenerate?: () => void;
    oninput?: () => void;
  } = $props();

  let shown = $state(false);
  let input: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (autofocus) input?.focus();
  });
</script>

<div class="field">
  <label for={id}>{label}</label>
  <div class="row">
    <div class="wrap">
      <input
        bind:this={input}
        {id}
        class="input mono"
        type={shown ? "text" : "password"}
        bind:value
        {disabled}
        {oninput}
        autocomplete="off"
        autocapitalize="off"
        spellcheck="false"
        aria-invalid={error ? "true" : undefined}
        aria-describedby={[hint && `${id}-hint`, error && `${id}-error`].filter(Boolean).join(" ") || undefined}
      />
      <button
        type="button"
        class="icon-btn reveal"
        aria-label={shown ? `Hide ${label.toLowerCase()}` : `Show ${label.toLowerCase()}`}
        aria-pressed={shown}
        title={shown ? "Hide" : "Show"}
        onclick={() => (shown = !shown)}
      >
        <Icon name={shown ? "eye-off" : "eye"} />
      </button>
    </div>
    {#if onGenerate}
      <button type="button" class="btn" onclick={() => { shown = true; onGenerate?.(); }} {disabled}>
        <Icon name="dice" /> Generate
      </button>
    {/if}
  </div>
  {#if error}
    <p class="error-text" id="{id}-error">{error}</p>
  {:else if hint}
    <p class="hint" id="{id}-hint">{hint}</p>
  {/if}
</div>

<style>
  .row {
    display: flex;
    gap: var(--s-2);
  }
  .wrap {
    position: relative;
    flex: 1;
    min-width: 0;
  }
  .input {
    padding-right: calc(var(--control-h) + var(--s-1));
    letter-spacing: 0.02em;
  }
  .reveal {
    position: absolute;
    top: 0;
    right: 0;
  }
  @media (max-width: 420px) {
    .row {
      flex-wrap: wrap;
    }
    .row > .btn {
      width: 100%;
    }
  }
</style>
