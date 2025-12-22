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

- Listen for the `messageQueue.Processed` (for XCM v3, v4 and v5) or `ump.ExecutedUpward` (for other XCM versions) system event on the Polkadot Relay Chain.
- What to index from the event:
    - **message_id**: A unique identifier for the message. This is crucial for tracing.
    - **origin**: The parachain_id from which the message originated along with message type.
    - **weight_used**: Weight info
    - **outcome**: The result of the execution (e.g., Success or Error).
    - Block number and extrinsic hash associated with the event for context.

**2. Downward Message Passing (DMP): Relay Chain to Parachain**

These are messages sent from the Relay Chain to a specific parachain.

- Listen for the `xcmPallet.limitedReserveTransferAssets`, `xcmPallet.limitedTeleportAssets`, `xcmPallet.reserveTransferAssets`, `xcmPallet.teleportAssets`, `xcmPallet.transferAssetsUsingTypeAndThen` extrinsic and/or `xcmPallet.Attempted`, `xcmPallet.Sent` event on the Polkadot Relay Chain. It contains origin, destination, message and message_id.

**3. Cross-Chain Message Passing (XCMP/HRMP): Parachain to Parachain**

These messages are routed through the Relay Chain but are not executed there. The Relay Chain acts as a transport layer.

- What to look for: The Relay Chain's role in XCMP is to pass messages from an origin parachain's upward queue to a destination parachain's downward queue. However, direct tracing is more efficient at the parachain level.

**xcmPallet.Attempted**

```
{
  "docs": "Execution of an XCM message was attempted.",
  "args_name": [
    "outcome"
  ],
  "args_type_name": [
    "xcm::latest::Outcome"
  ]
}
```

```
{
  "docs": "Execution of an XCM message was attempted.\n\n\\[ outcome \\]",
  "args_name": [
    ""
  ],
  "args_type_name": [
    "xcm::latest::Outcome"
  ]
}
```

**ump.ExecutedUpward**

```
{
  "docs": "Upward message executed with the given outcome.\n\\[ id, outcome \\]",
  "args_name": [
    "",
    ""
  ],
  "args_type_name": [
    "MessageId",
    "Outcome"
  ]
}
```

**ump.UpwardMessagesReceived**

```
{
  "docs": "Some upward messages have been received and will be processed.\n\\[ para, count, size \\]",
  "args_name": [
    "",
    "",
    ""
  ],
  "args_type_name": [
    "ParaId",
    "u32",
    "u32"
  ]
}
```

**messageQueue.Processed** (Introduced in v1000001)

```
{
  "docs": "Message is processed.",
  "args_name": [
    "id",
    "origin",
    "weight_used",
    "success"
  ],
  "args_type_name": [
    "[u8; 32]",
    "MessageOriginOf",
    "Weight",
    "bool"
  ]
}
```

```(after v1003003)
{
  "docs": "Message is processed.",
  "args_name": [
    "id",
    "origin",
    "weight_used",
    "success"
  ],
  "args_type_name": [
    "H256",
    "MessageOriginOf",
    "Weight",
    "bool"
  ]
}
```

## Polkadot Asset Hub

- **Asset Hub to Parachain/Relay**
    - Listen for `polkadotXcm.limitedReserveTransferAssets`, `polkadotXcm.limitedTeleportAssets`, `polkadotXcm.send`, `polkadotXcm.reserveTransferAssets`, `polkadotXcm.teleportAssets`, `polkadotXcm.transferAssets`, `polkadotXcm.transferAssetsUsingTypeAndThen` extrinsics. It will info about source, destination, assets etc. Listen for `polkadotXcm.Attempted` event. Find other events for this extrinsic and look for `parachainSystem.UpwardMessageSent`. For reserve transfers, event `xcmpqueue.XcmpMessageSent` would have message hash.

    - **When destination chain is Relay**
        - For XCM V2, not sure how to find corelation key. For XCM v3 and above, `parachainSystem.UpwardMessageSent` would have `message_hash` which acts as corelation key.

    - **When destination chain is any system parachain**
        - For XCM v3 and above, `xcmpqueue.XcmpMessageSent.UpwardMessageSent` would have `message_hash` which acts as corelation key. Also, the fact is, chains which we are targeting used XCM v3 or above only so no need to check what would be corelation key for below XCM v3.

- **Parachain/Relay to Asset Hub**
    - Listen for `messageQueue.processed` (for XCM v4 and v5) or `Dmpqueue.ExecutedDownward` (for older XCM versions) event

    - **When source was Relay chain**
        - For XCM v2, `dmp.ExecutedDownward` has `message_id` which acts as corelation key. For XCM v3, `message_hash` in `dmp.ExecutedDownWard` is corelation key. For XCM v4 and above, `messageQueue.Processed` has `message_id` but it's not corelation key.

    - **When source was any system chain**
        - For XCM v3, event `xcmpQueue.Success` has `message_hash` which is the key.
        - else `messageQueue.Processed` has `message_id` but it's not corelation key. However, on source chain, `polkadotXcm.Sent` event has `message_id` in it's args which is exactly same as dest. `message_id`

## Polkadot Bridge Hub

- **Bridge Hub to Parachain/Relay**
    - Listen for `polkadotXcm.Attempted` (when destination is Relay) and `xcmpqueue.XcmpMessageSent` (when destination is other system chain) (which hash `message_hash` directly in it) event. Find other events for this extrinsic and look for `parachainSystem.UpwardMessageSent` which hash `message_hash`.

- **Parachain/Relay to Bridge Hub**
    - Listen for `messageQueue.processed` (for XCM v4 and v5) or `Dmpqueue.ExecutedDownward` (for older XCM versions) event. Both events have message id or hash in it which would act as co-relation key.

## Polkadot Collectives

- **Collectives to Parachain/Relay**
    - When destination is Relay
        - Listen for `polkadotXcm.Attempted`. Find other events in extrinsic and get `message_hash` from `parachainSystem.UpwardMessageSent` event. But for XCM v2, not sure how to get `message_hash` because no `parachainSystem.UpwardMessageSent` event for XCM v2.

    - When destination is system chain
        - Listen for event `xcmpqueue.XcmpMessageSent`, would have `message_hash` which is corelation key.

- **Parachain/Relay to Collectives**
    - If source chain is Polkadot Relay, Listen for `messageQueue.processed` (for XCM v4 and v5, and not sure how to get corelation key for this) or `Dmpqueue.ExecutedDownward` (`message_id` is corelation key for older XCM versions) event.
    - else listen for `messageQueue.processed` (for XCM v4 and v5, and not sure for key) or `xcmpqueue.success` (`message_hash` is key, for other XCM versions).

## Polkadot Coretime

- **Coretime to Parachain/Relay**
    - There are many extrinsics like `polkadotXcm.limitedReserveTransferAssets`, `polkadotXcm.limitedTeleportAssets`, `polkadotXcm.send`, `polkadotXcm.reserveTransferAssets`, `polkadotXcm.teleportAssets`, `polkadotXcm.transferAssets`, `polkadotXcm.transferAssetsUsingTypeAndThen` (can be more) can initiate XCM transfers which will have info about source, destination, assets etc but event `parachainsystem.UpwardMessageSent` (when destination is Relay chain) or `xcmpqueue.XcmpMessageSent` (when destination in System chain) would have message hash which would act as co-relation key.

- **Parachain/Relay to Coretime**
    - Listen for `messageQueue.processed`. Event have message id in it which might act as co-relation key.

`parachainsystem.UpwardMessageSent`: Stays same for all runtimes

```
{
  messageHash: {
    type: 'Some',
    value: '0x54e2d0ea97d34f069b8b0d79a0e5ba4901946b065fb9a102aa094888de4429cb'
  }
}
```

`xcmpqueue.XcmpMessageSent`: Stays same for all runtimes

```
{
  messageHash: '0x494920be50870cc9623154a2ae42519175272c0d1f4b3ad9dd54792cf21a57c9'
}
```

`messageQueue.Processed`: Stays same for all runtime

```
{
  id: '0x894f9b5b5c7f0fdad7c87affee713c59f41d9d4bb21018a1fe669789100a00c8',
  origin: { type: 'Sibling', value: '1000' },
  success: true,
  weightUsed: { proofSize: '3465', refTime: '32930000' }
}
```

```
{
  id: '0x4a00b223eb85be70f7aec13fc8cdc6e9c3203f1fe4e8352e59ec7a85ae1e28dc',
  origin: { type: 'Parent', value: [] },
  success: true,
  weightUsed: { proofSize: '0', refTime: '123620000' }
}
```

## Polkadot People

- **People to Parachain/Relay**
    - There are many extrinsics like `polkadotXcm.limitedReserveTransferAssets`, `polkadotXcm.limitedTeleportAssets`, `polkadotXcm.send`, `polkadotXcm.reserveTransferAssets`, `polkadotXcm.teleportAssets`, `polkadotXcm.transferAssets`, `polkadotXcm.transferAssetsUsingTypeAndThen` (can be more) can initiate XCM transfers which will have info about source, destination, assets etc but event `parachainsystem.UpwardMessageSent` (when destination is Relay chain with `polkadotXcm.Attempted`) or `xcmpqueue.XcmpMessageSent` (when destination in System chain) would have message hash which would act as co-relation key. We have to find associated extrinsic with it as well.

- **Parachain/Relay to People**
    - Listen for `messageQueue.processed`. Works for all XCM versions. Event have message id in it which might act as co-relation key.

### Caveat

- [ ] Mostly, we are tracking XCM on chain with events, but important thing is, we should know which extrinsic emitted the event. Need to properly maintain event->extrinsic mapping.
    - This could be made with Crystal APIs itself. For every block, we can find all events (with `/blocks/:block_ref/events`). In response, we get extrinsic hash. All other events emitted from the extrinsic can be found with `/extrinsics/:extrinsic_hash/events` and more details about extrinsic itself can be found with `/extrinsics/:extrinsic_hash`. With this, we can properly establish event->extrinsic mapping.

- [ ] Most of the `messageQueue.processed` has a `id` parameter in it, but it does not match with actual corelation key which we can use to find source chain XCM details. Need to look into it.

These are few answers, we will look for next.
