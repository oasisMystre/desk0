import z from "zod";
import { agentOsSchema } from "@workspace/datasource";

export const generateAgentSchema = z.object({
  agent: z.uuid(),
  version: z.number(),
  organization: z.uuid(),
  os: z.array(agentOsSchema),
});

export const jobSchema = z.union([
  z.object({
    type: z.literal("build-agent"),
    payload: generateAgentSchema,
  }),
]);

export type Job = z.infer<typeof jobSchema>;
export type AgentOs = z.infer<typeof agentOsSchema>;
export type GenerateAgentPayload = z.infer<typeof generateAgentSchema>;
