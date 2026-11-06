import { pgTable, text, timestamp, unique, uuid } from "drizzle-orm/pg-core";

import { users } from "./users";

export const organizations = pgTable(
  "organizations",
  {
    id: uuid().defaultRandom().primaryKey(),
    name: text(),
    user: uuid().references(() => users.id, { onDelete: "cascade" }),
    createdAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
  },
  (columns) => [unique().on(columns.user, columns.name).nullsNotDistinct()],
);
