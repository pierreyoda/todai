/** What the rest of the frontend may use of domains/notes. */
export {
  monthNotesQueryOptions,
  noteKeys,
  noteMonthsQueryOptions,
  noteQueryOptions,
} from "./api/queries";
export { notePath } from "./model/paths";
export type { Note } from "./model/types";
export { noteDrafts } from "./state/noteDrafts.svelte";
export { default as NoteMarker } from "./ui/NoteMarker.svelte";
export { default as NotePanel } from "./ui/NotePanel.svelte";
