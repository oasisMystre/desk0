import { jsonb, pgTable, text, timestamp, uuid } from "drizzle-orm/pg-core";

import { organizations } from "./organizations";

type DeviceMetadata = {
  _version: 1,
  size?: {
    width: number,
    height: number
 }
}

export const devices = pgTable("devices", {
  id: uuid().defaultRandom().primaryKey(),
  os: text(),
  hostname: text(),
  username: text(),
  version: text(),
  metadata: jsonb().$type<DeviceMetadata>(),
  organization: uuid()
    .references(() => organizations.id, { onDelete: "cascade" })
    .notNull(),
  lastLoggedIn: timestamp({ withTimezone: true }).defaultNow().notNull(),
  createdAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
});
