import { Pool } from "pg";
import { drizzle } from "drizzle-orm/node-postgres";

import { relations } from "./relations";

export const createDB = (url: string) => {
  const pool = new Pool({ connectionString: url });

  return drizzle({ client: pool, relations });
};

export * from "./zod";
export * from "./schema";
export type Database = ReturnType<typeof createDB>;
