import { pgEnum } from "drizzle-orm/pg-core";

export const os = pgEnum("os", ["windows", "macos", "linux", "android", "ios"]);
