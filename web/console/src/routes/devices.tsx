import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/devices")({
	component: Devices,
});

function Devices() {
	return <div>Hello "/devices"!</div>;
}
