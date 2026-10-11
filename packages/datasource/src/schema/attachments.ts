import {
  boolean,
  jsonb,
  pgTable,
  text,
  timestamp,
  uuid,
} from "drizzle-orm/pg-core";

import { organizations } from "./organizations";

export const attachments = pgTable("attachments", {
  id: uuid().defaultRandom().primaryKey(),
  key: text().notNull(),
  uploaded: boolean().notNull(),
  contentType: text().notNull(),
  filename: text().notNull(),
  metadata: jsonb(),
  organization: uuid().references(() => organizations.id, {
    onDelete: "cascade",
  }),
  createdAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
});
