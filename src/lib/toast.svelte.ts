export type Tone = "info" | "ok" | "caution" | "danger";

export const toasts = $state<{ id: number; text: string; tone: Tone }[]>([]);

let next = 1;

export function notify(text: string, tone: Tone = "info", ms = 4000) {
  const id = next++;
  toasts.push({ id, text, tone });
  setTimeout(() => {
    const i = toasts.findIndex((t) => t.id === id);
    if (i >= 0) toasts.splice(i, 1);
  }, ms);
}
