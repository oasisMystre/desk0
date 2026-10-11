import "dotenv/config";
import path from "path";
import dotenv from "dotenv";
import { format } from "util";
import { execSync } from "child_process";

try {
  const root = execSync("git rev-parse --show-toplevel").toString().trim();
  if (root) dotenv.config({ path: path.resolve(root, ".env") });
} catch {}

type RefineFn<T> = <K>(value: K) => T | boolean;
type Env =
  | "S3_BUCKET"
  | "S3_ENDPOINT"
  | "S3_PUBLIC_BASE_URL"
  | "S3_ACCESS_KEY"
  | "S3_ACCESS_KEY_ID"
  | "DATABASE_URL"
  | "REDIS_URL"
  | "REDIS_USERNAME"
  | "DESK0_VERSION"
  | "REDIS_PASSWORD"
  | "REDIS_MASTER_NAME"
  | "REDIS_SENTINEL_PORT"
  | "REDIS_SENTINEL_HOSTNAME";

export function getEnv<T>(name: Env, nullable: true): T | undefined;
export function getEnv<T>(name: Env, refine?: RefineFn<T>): T;
export function getEnv<T>(name: Env, arg?: true | RefineFn<T>): T | undefined {
  const value = process.env[name] || process.env[`APP_${name}`];

  const isNullable = arg === true;
  const refine = typeof arg === "function" ? arg : undefined;

  if (value !== undefined) {
    try {
      const parsed: unknown = JSON.parse(value);
      return refine ? (refine(parsed) as T) : (parsed as T);
    } catch {
      return refine ? (refine(value) as T) : (value as unknown as T);
    }
  }

  if (!isNullable) throw new Error(format("%s not found in env file", name));

  return undefined;
}
