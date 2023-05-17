# Minecraft Rust Utilities

Yet another project that I'm using as an exuse to learn Rust

All utilities in this repo will be used to create a minecraft server scanning tool

Based off of an old Nodejs project of mine

Everything is (badly) implemented asynchronously with Tokio

---

### Currently Implemented Features:

-   RCON Client
-   -   RCON Packet data type
-   -   Easy log-in and command sending
-   Server List Pinger
-   -   One-function pinging
-   Check the auth status of servers
-   Bedrock Edition Server List Ping
-   Legacy protocol support

### Planned Features:

-   Join online mode and offline mode servers
-   Implement forge and fabric protocols to join modded servers
-   Control panel / gui
-   Bedrock Edition Player List (if possible)

### Maybe Goals:

These are things that I would like to have, but are likely outside the scope of this project

-   Custom TCP implementation
