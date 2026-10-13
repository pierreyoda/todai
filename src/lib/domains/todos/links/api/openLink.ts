import { openUrl } from "@tauri-apps/plugin-opener";

import { isWebUrl } from "../model/links";

/** Opens the http(s) `url` in the default browser, never in the app. Anything else is ignored (and logged). */
export const openLink = (url: string) => {
  if (!isWebUrl(url)) {
    console.error(`Not opening ${url}: not an http(s) link`);
    return;
  }
  openUrl(url).catch((error) => console.error(`Could not open ${url}:`, error));
};
