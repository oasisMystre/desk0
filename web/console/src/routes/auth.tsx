import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/auth")({ component: Auth });

function Auth() {
	return <div></div>;
}
