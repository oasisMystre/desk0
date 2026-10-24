CREATE TABLE "refresh_tokens" (
	"id" uuid DEFAULT gen_random_uuid() NOT NULL,
	"session" uuid NOT NULL,
	"token" bytea NOT NULL,
	"revokedAt" timestamp with time zone DEFAULT now(),
	"consumedAt" timestamp with time zone DEFAULT now(),
	"expiresAt" timestamp with time zone DEFAULT now() NOT NULL,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "sessions" (
	"id" uuid DEFAULT gen_random_uuid() NOT NULL,
	"user" uuid NOT NULL,
	"userAgent" text,
	"ipAddress" inet,
	"lastUsedAt" timestamp with time zone DEFAULT now(),
	"revokedAt" timestamp with time zone DEFAULT now(),
	"expiresAt" timestamp with time zone DEFAULT now() NOT NULL,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "refresh_tokens" ADD CONSTRAINT "refresh_tokens_session_sessions_id_fkey" FOREIGN KEY ("session") REFERENCES "sessions"("id") ON DELETE CASCADE;--> statement-breakpoint
ALTER TABLE "sessions" ADD CONSTRAINT "sessions_user_users_id_fkey" FOREIGN KEY ("user") REFERENCES "users"("id") ON DELETE CASCADE;