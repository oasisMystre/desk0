import path from "path";
import { tmpdir } from "os";
import { format } from "util";
import { randomUUID } from "crypto";
import { createReadStream, existsSync } from "fs";
import { spawn } from "child_process";
import { Upload } from "@aws-sdk/lib-storage";
import { DeleteObjectCommand, type S3Client } from "@aws-sdk/client-s3";
import { copyFile, mkdir, mkdtemp, rm, stat, writeFile } from "fs/promises";
import { agentAssets, attachments, type Database } from "@workspace/datasource";

import type { GenerateAgentPayload } from "./types";
import { getEnv } from "./env";
import assert from "assert";

export type MsiArchitecture = "x64" | "arm64";

const config = `
version = %d
agent = %s
organization = %s
`;

const tomlString = (value: string): string => {
  return JSON.stringify(value);
};

export const buildMsi = (
  source: string,
  output: string,
  cwd: string,
  arch: MsiArchitecture,
  publishDir?: string,
) => {
  return new Promise<void>((resolve, reject) => {
    const errorLogs: string[] = [];
    const args = ["build", source, "-arch", arch];
    if (publishDir) args.push("-d", format("PublishDir=%s", publishDir));

    args.push("-o", output, "-v");

    const child = spawn("wix", args, {
      cwd,
      shell: true,
      stdio: ["ignore", "pipe", "pipe"],
    });

    const log = (chunk: Buffer) => {
      for (const line of chunk.toString().split(/\r?\n/).filter(Boolean)) {
        console.log(format("[Wix] %s", line));
        errorLogs.push(line);
      }
    };

    child.stdout.on("data", log);
    child.stderr.on("data", log);
    child.once("error", reject);
    child.once("close", (code) => {
      if (code === 0) resolve();
      else
        reject(
          new Error(
            format(
              "Wix exited with code %d%s",
              code,
              errorLogs.length > 0
                ? format(": %s", errorLogs.join("\n"))
                : String(),
            ),
          ),
        );
    });
  });
};

export const bundleWindowAgentAsMsi = async (
  buildDir: string,
  arch: MsiArchitecture,
  payload: Omit<GenerateAgentPayload, "os">,
) => {
  const os = "windows";
  const archBuildDir = path.join(buildDir, arch);
  const distDir = path.join(archBuildDir, "dist");
  const output = path.join(distDir, format("Desk0-Agent-%s.msi", arch));
  const installerWxs = path.join(distDir, "installer.wxs");

  await mkdir(distDir, { recursive: true });

  const runner = async () => {
    const buildDirCandidates = [
      path.resolve(process.cwd(), "build"),
      path.resolve(import.meta.dirname, "../../../build"),
    ];

    const localBuildDir = buildDirCandidates.find((path) => existsSync(path));
    assert(localBuildDir, "build directory not found");

    const templateWxs = path.join(localBuildDir, "installer.wxs");
    const exeSource = path.join(
      localBuildDir,
      "latest",
      os,
      format("Desk0-Agent-%s.exe", arch),
    );

    await Promise.all([
      copyFile(templateWxs, installerWxs),
      copyFile(exeSource, path.join(distDir, "desk0-agent.exe")),
      writeFile(
        path.join(distDir, "config.toml"),
        format(
          config,
          payload.version,
          tomlString(payload.agent),
          tomlString(payload.organization),
        ),
        "utf-8",
      ),
    ]);

    await buildMsi(installerWxs, output, archBuildDir, arch, distDir);
  };

  await runner();
  return { arch, output };
};

export const buildWindowAgent = async (
  db: Database,
  s3: S3Client,
  s3Bucket: string,
  payload: Omit<GenerateAgentPayload, "os"> & { os: ["windows"] },
) => {
  const uploadedKeys = new Set<string>();
  const abortController = new AbortController();
  const buildDir = await mkdtemp(path.join(tmpdir(), "desk0-build-"));

  const runner = async (
    outputs: {
      arch: MsiArchitecture;
      output: string;
    }[],
  ) => {
    const id = randomUUID();
    const uploadedFiles = await Promise.all(
      outputs.map(async ({ arch, output }) => {
        const key = format(
          "agents/%s/%d/%s/Desk0-Agent-%s.msi",
          payload.organization,
          payload.version,
          id,
          arch,
        );

        const fileInfo = await stat(output);
        await new Upload({
          client: s3,
          params: {
            Key: key,
            Bucket: s3Bucket,
            Body: createReadStream(output),
            ContentLength: fileInfo.size,
            ContentType: "application/x-msi",
            Metadata: {
              agent: payload.agent,
              version: String(payload.version),
              organization: payload.organization,
            },
          },
          abortController,
          leavePartsOnError: false,
        }).done();

        uploadedKeys.add(key);

        return { key, arch };
      }),
    );

    return db.transaction(async (db) => {
      const attachmentRecords = await db
        .insert(attachments)
        .values(
          uploadedFiles.map(({ arch, key }) => ({
            key,
            uploaded: true,
            organization: payload.organization,
            contentType: "application/x-msi",
            filename: format("Desk0-Agent-%s.msi", arch),
            metadata: {
              arch,
              os: "windows",
              type: "agent",
            },
          })),
        )
        .returning({ id: attachments.id, metadata: attachments.metadata });

      return db
        .insert(agentAssets)
        .values(
          attachmentRecords.map((attachment) => ({
            agent: payload.agent,
            attachment: attachment.id,
            os: "windows" as const,
            version: getEnv<number>("DESK0_VERSION"),
            arch: (attachment.metadata as { arch: MsiArchitecture }).arch,
          })),
        )

        .returning();
    });
  };

  return Promise.all(
    (["arm64", "x64"] as const).map((arch) =>
      bundleWindowAgentAsMsi(buildDir, arch, payload),
    ),
  )
    .then((outputs) => runner(outputs))
    .catch(async (error) => {
      abortController.abort();
      if (uploadedKeys.size > 0) {
        await Promise.all(
          [...uploadedKeys].map((key) =>
            s3
              .send(new DeleteObjectCommand({ Key: key, Bucket: s3Bucket }))
              .catch((s3Error) => {
                console.error("Failed to remove orphaned R2 object", {
                  key,
                  error,
                  s3Error,
                });
              }),
          ),
        );
      }
      console.error(error);
      return Promise.reject(error);
    })
    .finally(() => rm(buildDir, { recursive: true, force: true }));
};
