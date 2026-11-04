import { foreignKey, pgTable, timestamp, uuid } from "drizzle-orm/pg-core";

import { devices } from "./devices";
import { applications } from "./applications";
import { organizations } from "./organizations";

export const deviceApplications = pgTable(
  "device_applications",
  {
    id: uuid().defaultRandom().primaryKey(),
    device: uuid().notNull(),
    application: uuid()
      .references(() => applications.id, { onDelete: "cascade" })
      .notNull(),
    organization: uuid()
      .references(() => organizations.id, { onDelete: "cascade" })
      .notNull(),
    createdAt: timestamp({ withTimezone: true }).notNull(),
  },
  (column) => [
    foreignKey({
      name: "device_applications_device_organization_fkey",
      columns: [column.device, column.organization],
      foreignColumns: [devices.id, devices.organization],
    }).onDelete("cascade"),
  ],
);
