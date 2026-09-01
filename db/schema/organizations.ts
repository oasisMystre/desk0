import { pgTable, text, timestamp, uuid } from "drizzle-orm/pg-core";

export const organizations = pgTable("organizations", {
  id: uuid().defaultRandom().primaryKey(),
  uid: text(),
  name: text(),
  createdAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
});
