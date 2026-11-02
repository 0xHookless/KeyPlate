<script lang="ts">
  let { bits, target, label }: { bits: number; target: number; label: string } = $props();

  const ratio = $derived(Math.max(0, Math.min(1, bits / target)));
  const tone = $derived(bits >= target ? "ok" : bits >= target * 0.6 ? "caution" : "danger");
  const verdict = $derived(bits === 0 ? "Empty" : bits >= target ? "Strong enough" : "Too easy to guess");
</script>

<div class="meter" data-tone={tone}>
  <div
    class="track"
    role="meter"
    aria-label={label}
    aria-valuemin={0}
    aria-valuemax={target}
    aria-valuenow={Math.round(Math.min(bits, target))}
    aria-valuetext="{Math.round(bits)} bits, {verdict.toLowerCase()}"
  >
    <div class="fill" style="transform: scaleX({ratio})"></div>
  </div>
  <p class="text"><span class="mono">≈ {Math.round(bits)} bits</span> · {verdict}</p>
</div>

<style>
  .meter {
    display: grid;
    gap: var(--s-1);
  }
  .track {
    height: 6px;
    border-radius: 3px;
    background: var(--surface-sunk);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    transform-origin: left;
    background: var(--danger);
    transition: transform var(--dur) var(--ease-out), background-color var(--dur) var(--ease-out);
  }
  [data-tone="caution"] .fill {
    background: var(--caution);
  }
  [data-tone="ok"] .fill {
    background: var(--ok);
  }
  .text {
    font-size: var(--text-sm);
    color: var(--text-3);
  }
</style>
