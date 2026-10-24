import { bytea, inet, pgTable, text, timestamp, uuid } from "drizzle-orm/pg-core";
import { users } from "./users";

export const sessions = pgTable("sessions", {
  id: uuid().defaultRandom().notNull(),
  user: uuid().references(() => users.id, { onDelete: "cascade" }).notNull(),
  userAgent: text(),
  ipAddress: inet(),
  lastUsedAt: timestamp({withTimezone: true}).defaultNow(),
  revokedAt: timestamp({ withTimezone: true }).defaultNow(),
  expiresAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
  createdAt: timestamp({withTimezone: true}).defaultNow().notNull(),
});

export const refreshTokens = pgTable("refresh_tokens", {
  id: uuid().defaultRandom().notNull(),
  session: uuid().references(() => sessions.id, { onDelete: "cascade" }).notNull(),
  token: bytea().notNull(),
  revokedAt: timestamp({ withTimezone: true }).defaultNow(),
  consumedAt: timestamp({ withTimezone: true }).defaultNow(),
  expiresAt: timestamp({ withTimezone: true }).defaultNow().notNull(),
  createdAt: timestamp({withTimezone: true}).defaultNow().notNull(),
});