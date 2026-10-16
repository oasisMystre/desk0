pub mod inbound;
pub mod outbound;

pub type Listener = ipc::IpcSharedMemory<outbound::IpcOutboundEvent, inbound::IpcInboundEvent>;
pub type Connector = ipc::IpcSharedMemory<inbound::IpcInboundEvent, outbound::IpcOutboundEvent>;
