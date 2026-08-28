# Introduction

Welcome to this project and guide.

To be clear right from the start: this is my first medium-to-large project in Rust, and my first time working with raw TCP sockets and binary wire protocols.

Most of my programming experience in terms of networking before this was high-level: making HTTP requests, working with JSON APIs, or using web frameworks where things like framing, buffers, and serialization are handled for you.

I wanted to learn how network protocols actually work underneath. What does a raw byte stream look like? How do you parse numbers without JSON? How do you build a game server without third-party frameworks?

To answer those questions, I wrote a Minecraft 1.12.2 (Protocol 340) server in Rust with one main constraint:

> Zero external dependencies. No `tokio`, no `serde`, no `byteorder`, no `nom`. Just Rust and `std`.

---

## What This Guide Is

When you look at protocol documentation on sites like wiki.vg, it is easy to get lost in huge tables of packet IDs and hex dumps. Large open-source server implementations can also be hard to learn from because they have thousands of files for world generation, plugins, and physics.

I wrote this documentation to serve two purposes:
1. A reference for how this codebase is laid out and how the pieces connect.
2. A step-by-step walkthrough showing how to build the server from an empty directory, explaining the logic, the bugs I ran into, and how things work on the wire.

If you are learning Rust or curious about network programming, you can read along and build this yourself.

---

## Structure of the Guide

The guide is split into four parts:

### Part I: Foundations & Networking
We start with networking fundamentals: how TCP byte streams work, why packets need length prefixes to avoid merging together, and how Minecraft uses variable-length integers (VarInt) and Big-Endian numbers.

### Part II: Architecture Reference
A walkthrough of the modules in this crate (`types`, `packet`, `connection`, `server`, and `error`), explaining the trait-based packet system and the connection state machine.

### Part III: Step-by-Step Build Guide
A hands-on tutorial starting from `cargo new`:
1. Building custom types and the VarInt encoder/decoder
2. Implementing packet framing and the `RawPacket` layer
3. Handling Handshakes and Server List Ping (MOTD and player count)
4. Authenticating players in offline mode
5. Sending the spawn packets to clear the "Downloading Terrain" screen
6. Handling multiple concurrent players using threads and atomic IDs

### Part IV: Verification & Beyond
Automated integration tests using real TCP sockets, plus next steps for terrain chunks, multiplayer synchronization, and physics.

---

## Prerequisites

You do not need prior network programming experience to follow along. You only need:
- A basic understanding of Rust (structs, enums, traits, pattern matching, and `Result`).
- A working Rust compiler (`rustc` and `cargo`).
- A copy of Minecraft Java Edition 1.12.2 if you want to test connecting with a real game client.
