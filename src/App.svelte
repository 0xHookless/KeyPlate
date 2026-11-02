<script lang="ts">
  import { onMount } from "svelte";
  import { api, onEvent, type LockReason, type Status } from "./lib/api";
  import { notify } from "./lib/toast.svelte";
  import Toasts from "./components/Toasts.svelte";
  import Welcome from "./screens/Welcome.svelte";
  import Unlock from "./screens/Unlock.svelte";
  import Vault from "./screens/Vault.svelte";

  let status = $state<Status | null>(null);
  let justCreated = $state(false);

  const lockMessages: Record<LockReason, string> = {
    idle: "Keyplate locked after a few idle minutes.",
    system: "Keyplate locked because Windows locked or went to sleep.",
    manual: "",
    replaced: "The vault file was replaced by a different vault, so Keyplate locked. Unlock to open the new file.",
  };

  onMount(() => {
    api.status().then((s) => (status = s));

    const unlisten = onEvent<LockReason>("vault-locked", async (reason) => {
      status = await api.status();
      justCreated = false;
      const message = lockMessages[reason];
      if (message) notify(message, reason === "replaced" ? "caution" : "info", 6000);
    });

    // Tell the Rust side someone is here, at most every 10 s, so auto-lock measures real idleness.
    let last = 0;
    const activity = () => {
      if (status?.stage !== "unlocked") return;
      const now = Date.now();
      if (now - last > 10_000) {
        last = now;
        api.touch().catch(() => {});
      }
    };
    window.addEventListener("pointerdown", activity);
    window.addEventListener("keydown", activity);
    return () => {
      unlisten.then((off) => off());
      window.removeEventListener("pointerdown", activity);
      window.removeEventListener("keydown", activity);
    };
  });
</script>

<Toasts />

{#if status}
  {#if status.stage === "welcome"}
    <Welcome
      onStatus={(s, created) => {
        status = s;
        justCreated = created;
      }}
    />
  {:else if status.stage === "locked"}
    <Unlock {status} onStatus={(s) => (status = s)} />
  {:else}
    <Vault {status} {justCreated} onStatus={(s) => (status = s)} />
  {/if}
{/if}
