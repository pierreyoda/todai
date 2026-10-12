import { redirect } from "@sveltejs/kit";

import { dateToTodaiDate, isTodaiDate } from "../../../../utils/dates";
import type { PageLoad } from "./$types";

// Another day's note, opened from the calendar: today's is Today's sub-page, `/today/notes`.
export const load: PageLoad = ({ params }) => {
  if (!isTodaiDate(params.day)) redirect(307, "/calendar");
  if (params.day === dateToTodaiDate(new Date())) redirect(307, "/today/notes");
  return { day: params.day };
};
