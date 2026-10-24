import { jsonb, pgTable, text, timestamp, uuid } from "drizzle-orm/pg-core";

import { attachments } from "./attachments";
import { organizations } from "./organizations";

type Scripts = {
  install?: {
    args: string[];
  };
  uninstall?: {
    args: string[];
  };
  run?: {
    args: string[];
  };
};

export const applications = pgTable("applications", {
  id: uuid().defaultRandom().notNull(),
  name: text().notNull(),
  image: uuid().references(() => attachments.id, { onDelete: "set null" }),
  asset: uuid()
    .references(() => attachments.id, { onDelete: "cascade" })
    .notNull(),
  organization: uuid()
    .references(() => organizations.id, { onDelete: "cascade" })
    .notNull(),
  scripts: jsonb().$type<Scripts>().notNull(),
  createdAt: timestamp({ withTimezone: true }).notNull(),
});
