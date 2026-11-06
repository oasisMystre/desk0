import {
  text,
  uuid,
  index,
  integer,
  pgTable,
  primaryKey,
} from "drizzle-orm/pg-core";

import { os } from "./enums";
import { agents } from "./agents";
import { attachments } from "./attachments";

export const agentAssets = pgTable(
  "agent_assets",
  {
    agent: uuid().references(() => agents.id, { onDelete: "cascade" }),
    os: os().notNull(),
    arch: text().notNull(),
    version: integer().notNull(),
    attachment: uuid()
      .references(() => attachments.id, {
        onDelete: "cascade",
      })
      .unique("unique_attachment"),
  },
  (column) => [
    primaryKey({
      columns: [column.agent, column.os, column.arch, column.version],
    }),
    index().on(column.agent),
  ],
);
