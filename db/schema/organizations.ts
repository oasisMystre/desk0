import { pgTable, text, uuid } from "drizzle-orm/pg-core";

export const organizations = pgTable("organizations", {
  id: uuid().defaultRandom().primaryKey(),
  name: text().notNull(),
});
