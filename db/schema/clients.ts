import { pgTable, text, timestamp, uuid } from "drizzle-orm/pg-core";

import { organizations } from "./organizations";

export const clients = pgTable("clients", {
  id: uuid().defaultRandom().primaryKey(),
  os: text(),
  hostname: text(),
  username: text(),
  version: text(),
  organization: uuid()
    .references(() => organizations.id, { onDelete: "cascade" })
    .notNull(),
  lastLoggedIn: timestamp({ withTimezone: true }).defaultNow().notNull(),
  createdAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
});
