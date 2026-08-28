# Summary

[Introduction](introduction.md)

# Part I: Foundations & Networking Fundamentals
- [1. Demystifying TCP & Stream Framing](part1/tcp_and_framing.md)
- [2. The Minecraft Wire Format & VarInts](part1/varint_and_primitives.md)
- [3. Composable Binary Codecs: McRead and McWrite](part1/serialization_traits.md)

# Part II: Architecture & System Reference
- [4. Project Architecture Overview](part2/architecture.md)
- [5. The Packet System & Wire Marshalling](part2/packet_system.md)
- [6. Connection Lifecycle & The State Machine](part2/state_machine.md)
- [7. Error Handling & Protocol Invariants](part2/error_handling.md)

# Part III: Hands-On Build Guide
- [8. Step 1: Laying the Groundwork & The Types Engine](part3/step1_types_and_errors.md)
- [9. Step 2: Packet Framing & The RawPacket Layer](part3/step2_framing.md)
- [10. Step 3: Handshaking & Server List Ping](part3/step3_handshake_and_status.md)
- [11. Step 4: Player Authentication & Offline Login](part3/step4_login.md)
- [12. Step 5: Spawning into the World (Play State)](part3/step5_play_and_spawn.md)
- [13. Step 6: Multi-Client Server & Concurrency](part3/step6_server_and_concurrency.md)

# Part IV: Verification & Beyond
- [14. Testing the Protocol & Integration Verification](part4/testing.md)
- [15. Where to Go Next: Worlds, Physics & Beyond](part4/where_to_go.md)
