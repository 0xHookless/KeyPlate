<script lang="ts">
  type Name =
    | "lock" | "plus" | "sliders" | "copy" | "eye" | "eye-off" | "edit" | "trash" | "back" | "search"
    | "print" | "download" | "folder" | "check" | "alert" | "key" | "close" | "dice" | "note" | "plate";

  let { name, size = 18 }: { name: Name; size?: number } = $props();

  const paths: Record<Name, string> = {
    lock: "M6 11h12v9H6z M8.5 11V8a3.5 3.5 0 0 1 7 0v3",
    plus: "M12 5v14 M5 12h14",
    sliders: "M4 7h9 M17 7h3 M4 17h3 M11 17h9 M15 5v4 M9 15v4",
    copy: "M9 9h10v11H9z M5 15V4h10",
    eye: "M2.5 12s3.5-6.5 9.5-6.5 9.5 6.5 9.5 6.5-3.5 6.5-9.5 6.5S2.5 12 2.5 12z M12 9.25a2.75 2.75 0 1 0 0 5.5 2.75 2.75 0 0 0 0-5.5z",
    "eye-off": "M3 3l18 18 M10.6 5.6A9.6 9.6 0 0 1 12 5.5c6 0 9.5 6.5 9.5 6.5a16 16 0 0 1-3 3.8 M6.4 6.9A15.6 15.6 0 0 0 2.5 12s3.5 6.5 9.5 6.5a9 9 0 0 0 4.3-1.1 M9.9 10.1a2.75 2.75 0 0 0 3.9 3.9",
    edit: "M4 20h4L19 9l-4-4L4 16z M13.5 6.5l4 4",
    trash: "M4 7h16 M9 7V4h6v3 M6.5 7l1 13h9l1-13 M10 11v6 M14 11v6",
    back: "M19 12H5 M11 6l-6 6 6 6",
    search: "M10.5 4a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13z M15.5 15.5L20 20",
    print: "M7 9V4h10v5 M7 17H4v-7h16v7h-3 M7 14h10v6H7z",
    download: "M12 4v11 M7 10l5 5 5-5 M5 20h14",
    folder: "M3 6h6l2 2h10v11H3z",
    check: "M5 12.5l4.5 4.5L19 7.5",
    alert: "M12 4L2.5 20h19z M12 10v4.5 M12 17.2v.3",
    key: "M14.5 3.5a6 6 0 1 1-4.2 10.2L4 20H2v-3l1.5-1.5H6V13h2.5l1.8-1.8A6 6 0 0 1 14.5 3.5z M16 8h.01",
    close: "M6 6l12 12 M18 6L6 18",
    dice: "M5 5h14v14H5z M9 9h.01 M15 9h.01 M12 12h.01 M9 15h.01 M15 15h.01",
    note: "M6 3h9l4 4v14H6z M14 3v5h5 M9 12h7 M9 16h5",
    plate: "M4 4h16v16H4z M8 8h2v2H8z M14 8h2v2h-2z M8 14h2v2H8z M14 14h2v2h-2z",
  };
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.75"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
>
  <path d={paths[name]} />
</svg>
