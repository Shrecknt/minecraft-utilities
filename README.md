# Minecraft Rust Utilities
Yet another project that I'm using as an exuse to learn Rust

All utilities in this repo will be used to create a minecraft server scanning tool

Based off of an old Nodejs project of mine

Everything is (badly) implemented asynchronously with Tokio

---

### Currently Implemented Features:
- RCON Client
- - RCON Packet data type
- - Easy log-in and command sending
- Server List Pinger
- - One-function pinging

### Planned Features:
- Join online mode and offline mode servers
- Implement forge and fabric protocols to join modded servers
- Control panel / gui

### Maybe Goals:
These are things that I would like to have, but are likely outside the scope of this project
- Legacy protocol support
- Custom TCP implementation
