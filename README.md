> [!Caution]
> **Misuse Disclaimer:** <br>
> The developers of Desk0 do not condone or support any unethical or illegal use of this software. Misuse, such as unauthorized access, control or invasion of privacy, is strictly against our guidelines. The authors are not responsible for any misuse of the application.

**Chat with us:** [Discord](https://discord.gg/G843aQkqmH) · [X](https://x.com/deskzer0) · [Telegram](https://t.me/desk0) · [YouTube](https://www.youtube.com/@desk0.zer0)

## TODO

### Security & Authentication

- [ ] Harden agent authentication and device enrollment security.
- [ ] Implement session-based authentication for agents.
- [ ] Replace per-use tokens with session-based agent and viewer tokens.
- [ ] Implement token expiration, revocation, and lifecycle management.

### Windows Agent

- [ ] Implement Session 0 process spawning using `CreateUserProcess` to launch IPC processes in Session 1.
- [ ] Handle interactive user session detection and lifecycle.
- [ ] Improve IPC communication between the Windows service and user sessions.

### Cross-platform Support

- [ ] Implement macOS service.
- [ ] Implement Linux service.
- [ ] Implement macOS and Linux logon/session capture.
- [ ] Handle user session lifecycle and unattended access across platforms.

### Management Console

- [ ] Build management console dashboard.
- [ ] Implement device management.
- [ ] Implement endpoint management.
- [ ] Add device enrollment and status monitoring.
- [ ] Add remote session monitoring and management.
