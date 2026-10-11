import {
  text,
  uuid,
  index,
  unique,
  integer,
  pgEnum,
  pgTable,
  timestamp,
  primaryKey,
} from "drizzle-orm/pg-core";

import { attachments } from "./attachments";
import { organizations } from "./organizations";

export const os = pgEnum("os", ["windows", "macos", "linux", "android", "ios"]);

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
