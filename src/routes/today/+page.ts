import { redirect } from "@sveltejs/kit";

// Today's todos are the home page: `/today` only holds its sub-pages, e.g. `/today/notes`.
export const load = () => redirect(307, "/");
