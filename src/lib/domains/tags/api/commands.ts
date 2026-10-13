import type { UUID } from "node:crypto";

import { createInvoker } from "#shared/ipc/invoke.ts";
import type { Tag } from "../model/types";

type TagCommands = {
  list_tags: {
    args?: never;
    returns: Tag[];
  };
  create_tag: {
    args: {
      name: string;
      /** Format: #RRGGBB. */
      color: string;
    };
    returns: Tag;
  };
  update_tag: {
    args: {
      id: UUID;
      name?: string;
      /** Format: #RRGGBB. */
      color?: string;
    };
    returns: Tag;
  };
  delete_tag: {
    args: {
      id: UUID;
    };
    returns: never;
  };
};

export const invokeTags = createInvoker<TagCommands>("command");
