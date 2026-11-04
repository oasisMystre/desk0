import { uuid, unique, integer, pgTable, timestamp } from "drizzle-orm/pg-core";

import { organizations } from "./organizations";

export const agents = pgTable(
  "agents",
  {
    id: uuid().defaultRandom().primaryKey(),
    organization: uuid()
      .references(() => organizations.id, { onDelete: "cascade" })
      .notNull(),
    version: integer().notNull(),
    createdAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
  },
  (column) => [
    unique().on(column.organization, column.version).nullsNotDistinct(),
  ],
);
