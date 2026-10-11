import { defineRelations } from "drizzle-orm";

import * as schema from "./schema";

export const relations = defineRelations(schema, (relation) => ({
  users: {
    organizations: relation.many.organizations(),
  },

  organizations: {
    agents: relation.many.agents(),
    devices: relation.many.devices(),
    attachments: relation.many.attachments(),
    user: relation.one.users({
      from: relation.organizations.user,
      to: relation.users.id,
    }),
  },
  attachments: {
    organization: relation.one.organizations({
      from: relation.attachments.organization,
      to: relation.organizations.id,
    }),
  },
  devices: {
    organization: relation.one.organizations({
      from: relation.devices.organization,
      to: relation.organizations.id,
    }),
  },

  agents: {
    agentAssets: relation.many.agentAssets(),
    organization: relation.one.organizations({
      from: relation.agents.organization,
      to: relation.organizations.id,
    }),
  },
  agentAssets: {
    attachment: relation.one.attachments({
      from: relation.agentAssets.attachment,
      to: relation.attachments.id,
    }),
  }
}));
