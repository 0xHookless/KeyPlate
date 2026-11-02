const DAY = 86_400_000;

export function ago(ms: number): string {
  const startOfToday = new Date().setHours(0, 0, 0, 0);
  if (ms >= startOfToday) return "today";
  if (ms >= startOfToday - DAY) return "yesterday";
  const days = Math.floor((startOfToday - ms) / DAY) + 1;
  if (days < 30) return `${days} days ago`;
  return new Date(ms).toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" });
}

export function kindLabel(kind: "seed" | "note"): string {
  return kind === "seed" ? "Seed phrase" : "Note";
}

export function plural(n: number, one: string, many = `${one}s`): string {
  return `${n} ${n === 1 ? one : many}`;
}
