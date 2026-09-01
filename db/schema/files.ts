import { pgTable, text, timestamp, uuid } from "drizzle-orm/pg-core";

import { organizations } from "./organizations";

export const files = pgTable("files", {
  id: uuid().defaultRandom().notNull(),
  name: text(),
  organization: uuid().references(() => organizations.id, {
    onDelete: "cascade",
  }),
  createdAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
});
