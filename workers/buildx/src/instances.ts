import { createDB } from "@workspace/datasource";

import { getEnv } from "./env";
import { createRedis } from "./redis";
import { S3Client } from "@aws-sdk/client-s3";

export const redis = createRedis();
export const db = createDB(getEnv("DATABASE_URL"));

export const s3 = new S3Client({
  region: getEnv("S3_REGION"),
  credentials: {
    accessKeyId: getEnv("S3_ACCESS_KEY_ID"),
    secretAccessKey: getEnv("S3_ACCESS_KEY"),
  },
  endpoint: getEnv("S3_ENDPOINT"),
});
export const s3Bucket = getEnv<string>("S3_BUCKET");
