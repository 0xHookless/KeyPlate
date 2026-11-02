// Browser-only stand-in for the Rust side, used for design previews (`npm run dev`).
// Holds public BIP39 test vectors only. Start at a given screen with ?stage=welcome|locked|unlocked.

import type { EntryDetail, EntryInput, SeedCheck, Settings, Status, Strength } from "./api";

export const mockEvents = new EventTarget();

const SAMPLE_WORDS = [
  "abandon", "about", "above", "absurd", "acoustic", "advice", "amount", "art", "avoid", "cage",
  "doctor", "legal", "letter", "sausage", "thank", "useful", "vote", "wave", "winner", "worth",
  "wrong", "year", "yellow", "zoo", "copper", "lantern", "orbit", "violin", "harbor", "ember",
];
const VALID = new Set([
  "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
  "legal winner thank year wave sausage worth useful legal winner thank yellow",
  "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
  "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
]);

const day = 86_400_000;
const now = Date.now();
let entries: EntryDetail[] = [
  {
    id: "demo-1",
    kind: "seed",
    title: "Hardware wallet (test vector)",
    words: "legal winner thank year wave sausage worth useful legal winner thank yellow".split(" "),
    bip39Passphrase: null,
    network: "Bitcoin",
    body: "Public BIP39 test vector, for previews only.",
    created: now - 40 * day,
    updated: now - 3 * day,
  },
  {
    id: "demo-2",
    kind: "seed",
    title: "Mobile wallet (test vector)",
    words: "letter advice cage absurd amount doctor acoustic avoid letter advice cage above".split(" "),
    bip39Passphrase: "demo-passphrase",
    network: "Ethereum",
    body: "",
    created: now - 20 * day,
    updated: now - 20 * day,
  },
  {
    id: "demo-3",
    kind: "note",
    title: "Exchange recovery codes",
    words: [],
    bip39Passphrase: null,
    network: null,
    body: "Placeholder text for the design preview.\nLine two of the note.",
    created: now - 10 * day,
    updated: now - 1 * day,
  },
];

const params = new URLSearchParams(location.search);
let settings: Settings = { autoLockMinutes: 5, clipboardSeconds: 30, practiceDays: 30 };
let stage = (params.get("stage") as Status["stage"]) ?? "welcome";
let folder: string | null = stage === "welcome" ? null : "G:\\My Drive\\Keyplate";
if (params.get("empty") === "1") entries = [];

const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
const fail = (code: string, message: string) => Promise.reject({ code, message });

function status(): Status {
  return { stage, folder, offline: params.get("offline") === "1", practiceDue: params.get("practice") === "1", settings };
}

function summary(e: EntryDetail) {
  return {
    id: e.id,
    kind: e.kind,
    title: e.title,
    network: e.network,
    wordCount: e.words.length,
    hasPassphrase: !!e.bip39Passphrase,
    hasBody: !!e.body,
    created: e.created,
    updated: e.updated,
    checksumValid: e.kind === "seed" ? VALID.has(e.words.join(" ")) : null,
  };
}

function bits(p: string): number {
  if (!p) return 0;
  const classes = [/[a-z]/, /[A-Z]/, /\d/, /[^a-zA-Z\d]/].filter((r) => r.test(p)).length;
  return Math.min(140, p.length * (1.6 + classes * 0.7));
}

function strength(first: string, second: string): Strength {
  const firstBits = bits(first);
  const secondBits = bits(second);
  const combinedBits = firstBits + secondBits;
  const problems: Strength["problems"] = [];
  if (!first) problems.push("firstEmpty");
  if (!second) problems.push("secondEmpty");
  if (first && first === second) problems.push("same");
  if (first && firstBits < 30) problems.push("firstWeak");
  if (second && secondBits < 30) problems.push("secondWeak");
  if (combinedBits < 80) problems.push("combinedWeak");
  return { firstBits, secondBits, combinedBits, acceptable: problems.length === 0, problems };
}

function checkSeed(words: string[]): SeedCheck {
  const norm = words.map((w) => w.trim().toLowerCase());
  const unknown = norm
    .map((w, index) => ({ w, index }))
    .filter(({ w }) => w && !/^[a-z]{3,8}$/.test(w))
    .map(({ w, index }) => ({ index, suggestions: SAMPLE_WORDS.filter((s) => s.startsWith(w.slice(0, 2))).slice(0, 4) }));
  if (unknown.length) return { status: "unknownWords", words: unknown };
  const filled = norm.filter(Boolean).length;
  if (filled !== norm.length || ![12, 15, 18, 21, 24].includes(filled)) return { status: "incomplete", filled };
  return VALID.has(norm.join(" ")) ? { status: "valid" } : { status: "badChecksum" };
}

export async function mockCall(cmd: string, a: Record<string, any>): Promise<unknown> {
  await wait(60);
  switch (cmd) {
    case "status":
      return status();
    case "drive_suggestions":
      return ["G:\\My Drive\\Keyplate"];
    case "pick_folder":
      return "D:\\Backups\\Keyplate";
    case "estimate_strength":
      return strength(a.first, a.second);
    case "generate_passphrase":
      return Array.from({ length: 5 }, () => SAMPLE_WORDS[Math.floor(Math.random() * SAMPLE_WORDS.length)]).join("-");
    case "create_vault":
      await wait(1400);
      folder = a.folder;
      stage = "unlocked";
      entries = [];
      return status();
    case "use_existing_vault":
      folder = a.folder;
      stage = "locked";
      return status();
    case "forget_vault":
      folder = null;
      stage = "welcome";
      return status();
    case "unlock":
      await wait(1100);
      if (a.first.includes("wrong") || a.second.includes("wrong") || !a.first || !a.second)
        return fail(
          "wrongPasswords",
          "These passwords don't open this vault. Check each one, and that the first password is in the first box.",
        );
      stage = "unlocked";
      return status();
    case "lock":
      stage = "locked";
      mockEvents.dispatchEvent(new CustomEvent("vault-locked", { detail: "manual" }));
      return status();
    case "touch":
      return null;
    case "list_entries":
      return entries.map(summary).sort((x, y) => x.title.localeCompare(y.title));
    case "reveal_entry":
      return structuredClone(entries.find((e) => e.id === a.id));
    case "save_entry": {
      const input = a.input as EntryInput;
      const t = Date.now();
      const existing = input.id ? entries.find((e) => e.id === input.id) : undefined;
      const words = input.kind === "seed" ? input.words.map((w) => w.trim().toLowerCase()).filter(Boolean) : [];
      const title = input.title.trim() || (input.kind === "seed" ? "Untitled seed phrase" : "Untitled note");
      if (existing) {
        Object.assign(existing, { ...input, id: existing.id, title, words, updated: t });
        return summary(existing);
      }
      const e: EntryDetail = { ...input, id: `demo-${t}`, title, words, created: t, updated: t };
      entries.push(e);
      return summary(e);
    }
    case "delete_entry":
      entries = entries.filter((e) => e.id !== a.id);
      return null;
    case "check_seed":
      return checkSeed(a.words);
    case "suggest_words":
      return SAMPLE_WORDS.filter((w) => w.startsWith(String(a.typed).toLowerCase())).slice(0, 6);
    case "copy_field":
      return settings.clipboardSeconds;
    case "export_backup":
      return "D:\\Backups\\keyplate-backup-20261102.keyplate";
    case "change_passwords":
      await wait(1800);
      return null;
    case "practice_unlock":
      await wait(900);
      return !a.first.includes("wrong") && !a.second.includes("wrong");
    case "save_settings":
      settings = a.settings;
      return status();
    case "recovery_info":
      return {
        folder,
        vaultFile: `${folder}\\vault.keyplate`,
        installerSha256: null,
        appVersion: "0.1.0",
        kdf: "Argon2id · 512 MiB · 4 passes · 4 lanes",
      };
    default:
      return fail("internal", `Unknown command ${cmd}`);
  }
}
