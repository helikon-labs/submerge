---
id: mycelium-spec
sidebar_position: 2
slug: /mycelium-spec
---

# Submerge Mycelium Specification

Mycelium is a data processing system that aggregates and indexes cross-chain messages (XCM) across the Polkadot ecosystem. It enables seamless interoperability, real-time tracking, and structured access to cross-chain data and transactions.

Each supported chain will be indexed individually and Crystal APIs have been created. This documentation outlines the intuition behind the system design and details the implementation logic for indexing and processing cross-chain messages.

To track XCM messages involving the Relay Chain, we need to monitor three types of message passing:

1.  **UMP (Upward Message Passing):** Messages from a parachain to the Relay Chain.
2.  **DMP (Downward Message Passing):** Messages from the Relay Chain to a parachain.
3.  **XCMP (Cross-Chain Message Passing):** Messages from one parachain to another, which are routed through the Relay Chain.

For each of these, we will monitor specific events and extrinsics to identify the XCM messages and track their lifecycle.

## Polkadot Relay Chain

**1. Upward Message Passing (UMP): Parachain to Relay Chain**

These are messages originating from a parachain and destined for the Relay Chain.

- Listen for the `messageQueue.Processed` system event on the Polkadot Relay Chain.
- What to index from the event:
    - **message_id**: A unique identifier for the message. This is crucial for tracing.
    - **origin**: The parachain_id from which the message originated along with message type.
    - **weight_used**: Weight info
    - **outcome**: The result of the execution (e.g., Success or Error).
    - Block number and extrinsic hash associated with the event for context.
- How to trace:
    - The message_id can be used to correlate this event with the original `polkadotXcm.Sent` event on the source parachain.
- Crystal API Query: GET /events?pallet=ump&event=ExecutedUpward

**2. Downward Message Passing (DMP): Relay Chain to Parachain**

These are messages sent from the Relay Chain to a specific parachain.

- Listen for the `xcmPallet` extrinsic and `xcmPallet.Attempted`, `xcmPallet.Sent` event on the Polkadot Relay Chain. It contains origin, destination, message and message_id.


**3. Cross-Chain Message Passing (XCMP/HRMP): Parachain to Parachain**

These messages are routed through the Relay Chain but are not executed there. The Relay Chain acts as a transport layer.

- What to look for: The Relay Chain's role in XCMP is to pass messages from an origin parachain's upward queue to a destination parachain's downward queue. However, direct tracing is more efficient at the parachain level.