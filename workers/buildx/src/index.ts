import { format } from "util";

import type { Job } from "./types";
import { db, redis, s3, s3Bucket } from "./instances";
import { buildWindowAgent } from "./build";

const GROUP_NAME = "node_workers";
const STREAM_NAME = "job_queue_stream";
const CONSUMER_NAME = format("worker_%s", process.pid);

/// improve this to retry failed jobs
async function main() {
  while (true) {
    const runner = async () => {
      const results = await redis.xreadgroup(
        "GROUP",
        GROUP_NAME,
        CONSUMER_NAME,
        "COUNT",
        1,
        "BLOCK",
        5000,
        "STREAMS",
        STREAM_NAME,
        ">",
      );

      if (results) {
        const [, messages] = results[0]!;

        const acks: string[] = [];
        for (const message of messages) {
          const jobId = message[0]!;
          const jobData = message[1]!;

          const payloadIndex = jobData.indexOf("payload") + 1;
          const job: Job = JSON.parse(jobData[payloadIndex]!);

          switch (job.type) {
            case "build-agent": {
              const payload = job.payload;
              await buildWindowAgent( db, s3, s3Bucket, {...payload, os: ["windows"]})
                .then(() => acks.push(jobId))
                .catch((error) => {
                  console.error("Job runner failed:", { jobId, error });
                });
              break;
            }
          }
        }

        await redis.xack(STREAM_NAME, GROUP_NAME, ...acks);
      }
    };

    await runner().catch((error) => {
      console.error("Error processing job:", error);
      return new Promise((resolve) => setTimeout(resolve, 1000));
    });
  }
}

main();
