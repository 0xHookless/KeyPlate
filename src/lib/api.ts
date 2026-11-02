// Typed access to the Rust side. In a plain browser (design previews) it falls back to an
// in-memory mock holding public BIP39 test vectors, never real data.

export type Stage = "welcome" | "locked" | "unlocked";
export type EntryKind = "seed" | "note";

export interface Settings {
  autoLockMinutes: number;
  clipboardSeconds: number;
  practiceDays: number;
}

export interface Status {
  stage: Stage;
  folder: string | null;
  offline: boolean;
  practiceDue: boolean;
  settings: Settings;
}

export interface EntrySummary {
  id: string;
  kind: EntryKind;
  title: string;
  network: string | null;
  wordCount: number;
  hasPassphrase: boolean;
  hasBody: boolean;
  created: number;
  updated: number;
  checksumValid: boolean | null;
}

export interface EntryDetail {
  id: string;
  kind: EntryKind;
  title: string;
  words: string[];
  bip39Passphrase: string | null;
  network: string | null;
  body: string;
  created: number;
  updated: number;
}

export interface EntryInput {
  id: string | null;
  kind: EntryKind;
  title: string;
  words: string[];
  bip39Passphrase: string | null;
  network: string | null;
  body: string;
}

export type Problem = "firstEmpty" | "secondEmpty" | "same" | "firstWeak" | "secondWeak" | "combinedWeak";

export interface Strength {
  firstBits: number;
  secondBits: number;
  combinedBits: number;
  acceptable: boolean;
  problems: Problem[];
}

export type SeedCheck =
  | { status: "valid" }
  | { status: "incomplete"; filled: number }
  | { status: "unknownWords"; words: { index: number; suggestions: string[] }[] }
  | { status: "badChecksum" };

export interface RecoveryInfo {
  folder: string;
  vaultFile: string;
  installerSha256: string | null;
  appVersion: string;
  kdf: string;
}

export interface UiError {
  code: string;
  message: string;
}

export type LockReason = "idle" | "system" | "manual" | "replaced";

export const MIN_EACH_BITS = 30;
export const MIN_COMBINED_BITS = 80;

export const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(cmd, args);
  }
  const { mockCall } = await import("./mock");
  return mockCall(cmd, args ?? {}) as Promise<T>;
}

export const api = {
  status: () => call<Status>("status"),
  driveSuggestions: () => call<string[]>("drive_suggestions"),
  pickFolder: () => call<string | null>("pick_folder"),
  estimateStrength: (first: string, second: string) => call<Strength>("estimate_strength", { first, second }),
  generatePassphrase: () => call<string>("generate_passphrase"),
  createVault: (folder: string, first: string, second: string) =>
    call<Status>("create_vault", { folder, first, second }),
  useExistingVault: (folder: string) => call<Status>("use_existing_vault", { folder }),
  forgetVault: () => call<Status>("forget_vault"),
  unlock: (first: string, second: string) => call<Status>("unlock", { first, second }),
  lock: () => call<Status>("lock"),
  touch: () => call<void>("touch"),
  listEntries: () => call<EntrySummary[]>("list_entries"),
  revealEntry: (id: string) => call<EntryDetail>("reveal_entry", { id }),
  saveEntry: (input: EntryInput) => call<EntrySummary>("save_entry", { input }),
  deleteEntry: (id: string) => call<void>("delete_entry", { id }),
  checkSeed: (words: string[]) => call<SeedCheck>("check_seed", { words }),
  suggestWords: (typed: string) => call<string[]>("suggest_words", { typed }),
  copyField: (id: string, field: "phrase" | "passphrase" | "body") => call<number>("copy_field", { id, field }),
  exportBackup: () => call<string | null>("export_backup"),
  changePasswords: (currentFirst: string, currentSecond: string, newFirst: string, newSecond: string) =>
    call<void>("change_passwords", { currentFirst, currentSecond, newFirst, newSecond }),
  practiceUnlock: (first: string, second: string) => call<boolean>("practice_unlock", { first, second }),
  saveSettings: (settings: Settings) => call<Status>("save_settings", { settings }),
  recoveryInfo: () => call<RecoveryInfo>("recovery_info"),
};

type EventName = "vault-locked" | "vault-changed";

export async function onEvent<T>(name: EventName, handler: (payload: T) => void): Promise<() => void> {
  if (isTauri) {
    const { listen } = await import("@tauri-apps/api/event");
    return listen<T>(name, (e) => handler(e.payload));
  }
  const { mockEvents } = await import("./mock");
  const fn = (e: Event) => handler((e as CustomEvent<T>).detail);
  mockEvents.addEventListener(name, fn);
  return () => mockEvents.removeEventListener(name, fn);
}

export function errorText(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) return String((e as UiError).message);
  return "Something went wrong. Try again.";
}

export function errorCode(e: unknown): string | null {
  return e && typeof e === "object" && "code" in e ? String((e as UiError).code) : null;
}
