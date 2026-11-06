CREATE TYPE "os" AS ENUM('windows', 'macos', 'linux', 'android', 'ios');--> statement-breakpoint
CREATE TABLE "users" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	"uid" text UNIQUE,
	"lastLoggedIn" timestamp with time zone DEFAULT now() NOT NULL,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "agents" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	"organization" uuid NOT NULL,
	"version" integer NOT NULL,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "agents_organization_version_unique" UNIQUE NULLS NOT DISTINCT("organization","version")
);
--> statement-breakpoint
CREATE TABLE "agent_assets" (
	"agent" uuid,
	"os" "os",
	"arch" text,
	"version" integer,
	"attachment" uuid CONSTRAINT "unique_attachment" UNIQUE,
	CONSTRAINT "agent_assets_pkey" PRIMARY KEY("agent","os","arch","version")
);
--> statement-breakpoint
CREATE TABLE "devices" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	"os" text,
	"hostname" text,
	"username" text,
	"version" text,
	"pubkey" bytea NOT NULL,
	"favorite" boolean DEFAULT false,
	"metadata" jsonb,
	"organization" uuid NOT NULL,
	"lastLoggedIn" timestamp with time zone DEFAULT now() NOT NULL,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "devices_id_organization_unique" UNIQUE("id","organization")
);
--> statement-breakpoint
CREATE TABLE "refresh_tokens" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	"session" uuid NOT NULL,
	"token" bytea NOT NULL,
	"revokedAt" timestamp with time zone DEFAULT now(),
	"consumedAt" timestamp with time zone DEFAULT now(),
	"expiresAt" timestamp with time zone DEFAULT now() NOT NULL,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "sessions" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	"user" uuid NOT NULL,
	"userAgent" text,
	"ipAddress" inet,
	"lastUsedAt" timestamp with time zone DEFAULT now(),
	"revokedAt" timestamp with time zone DEFAULT now(),
	"expiresAt" timestamp with time zone DEFAULT now() NOT NULL,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "attachments" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	"key" text NOT NULL,
	"uploaded" boolean NOT NULL,
	"contentType" text NOT NULL,
	"filename" text NOT NULL,
	"metadata" jsonb,
	"organization" uuid,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "organizations" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	"name" text,
	"user" uuid,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "organizations_user_name_unique" UNIQUE NULLS NOT DISTINCT("user","name")
);
--> statement-breakpoint
CREATE TABLE "applications" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	"name" text NOT NULL,
	"os" "os" NOT NULL,
	"image" uuid,
	"asset" uuid NOT NULL,
	"organization" uuid NOT NULL,
	"scripts" jsonb NOT NULL,
	"createdAt" timestamp with time zone NOT NULL
);
--> statement-breakpoint
CREATE TABLE "device_applications" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	"device" uuid NOT NULL,
	"application" uuid NOT NULL,
	"organization" uuid NOT NULL,
	"createdAt" timestamp with time zone NOT NULL
);
--> statement-breakpoint
CREATE INDEX "agent_assets_agent_index" ON "agent_assets" ("agent");--> statement-breakpoint
ALTER TABLE "agents" ADD CONSTRAINT "agents_organization_organizations_id_fkey" FOREIGN KEY ("organization") REFERENCES "organizations"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "agent_assets" ADD CONSTRAINT "agent_assets_agent_agents_id_fkey" FOREIGN KEY ("agent") REFERENCES "agents"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "agent_assets" ADD CONSTRAINT "agent_assets_attachment_attachments_id_fkey" FOREIGN KEY ("attachment") REFERENCES "attachments"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "devices" ADD CONSTRAINT "devices_organization_organizations_id_fkey" FOREIGN KEY ("organization") REFERENCES "organizations"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "refresh_tokens" ADD CONSTRAINT "refresh_tokens_session_sessions_id_fkey" FOREIGN KEY ("session") REFERENCES "sessions"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "sessions" ADD CONSTRAINT "sessions_user_users_id_fkey" FOREIGN KEY ("user") REFERENCES "users"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "attachments" ADD CONSTRAINT "attachments_organization_organizations_id_fkey" FOREIGN KEY ("organization") REFERENCES "organizations"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "organizations" ADD CONSTRAINT "organizations_user_users_id_fkey" FOREIGN KEY ("user") REFERENCES "users"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "applications" ADD CONSTRAINT "applications_image_attachments_id_fkey" FOREIGN KEY ("image") REFERENCES "attachments"("id") ON DELETE SET NULL;--> statement-breakpoint
ALTER TABLE "applications" ADD CONSTRAINT "applications_asset_attachments_id_fkey" FOREIGN KEY ("asset") REFERENCES "attachments"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "applications" ADD CONSTRAINT "applications_organization_organizations_id_fkey" FOREIGN KEY ("organization") REFERENCES "organizations"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "device_applications" ADD CONSTRAINT "device_applications_application_applications_id_fkey" FOREIGN KEY ("application") REFERENCES "applications"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "device_applications" ADD CONSTRAINT "device_applications_organization_organizations_id_fkey" FOREIGN KEY ("organization") REFERENCES "organizations"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "device_applications" ADD CONSTRAINT "device_applications_device_organization_fkey" FOREIGN KEY ("device","organization") REFERENCES "devices"("id","organization") ON DELETE CASCADE;