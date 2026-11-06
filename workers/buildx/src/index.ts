import { Worker } from "bullmq";
import { formatError } from "zod";

import { createRedis } from "./redis";
import { createJobLogger } from "./utils";
import { buildWindowAgent } from "./build";
import { generateAgentSchema } from "./types";
import { db, s3, s3Bucket } from "./instances";

const worker = new Worker(
  "desk0",
  async (job) => {
    const logger = createJobLogger(job.log);

    if (job.name === "build-agent") {
      const result = generateAgentSchema.safeParse(JSON.parse(job.data));

      if (result.success) {
        const payload = result.data;
        const results = [];

        for (const os of payload.os) {
          switch (os) {
            case "windows":
              results.push(
                buildWindowAgent(
                  db,
                  s3,
                  s3Bucket,
                  {
                    ...payload,
                    os: ["windows"],
                  },
                  logger,
                ),
              );
          }
        }

        return await Promise.all(results).catch((error) => {
          logger.error("Job failed", { error, job: job.id });
          return Promise.reject(error);
        });
      }

      return { error: formatError(result.error) };
    }
  },
  { connection: createRedis({ maxRetriesPerRequest: null }) },
);

worker.on("completed", ({ returnvalue }) => {
  console.log(returnvalue);
});
