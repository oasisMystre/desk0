import z from "zod";
import { os } from "./schema";

export const agentOsSchema = z.enum(os.enumValues);
