# Deterministic Perpetual Order Book

A minimal off-chain perpetual order book with deterministic FIFO matching and full replayability.

This project implements a single-market perpetual matching engine that guarantees identical state reconstruction when replaying the same event log.

This implementation focuses on:

- Correct FIFO matching

- Explicit state mutation

- Deterministic replay

- Clear architectural separation

The implementation is written in **Rust**.

## Language Choice: Rust

Rust was chosen for the following reasons:

### 1. Deterministic execution characteristics

- No garbage collection pauses

- Explicit memory management

### 2. Strong type system

- Prevents invalid state transitions

- Enforces correctness at compile time

### 3. Zero-cost abstractions

- Suitable for exchange-style systems

### 4. Safe concurrency

- Used for a lock-free SPSC logging pipeline

Rust provides predictable execution and explicit ownership semantics, both critical for deterministic replay systems.
Given that the role is for Rust, it is more natural that the language choice mirrors that requirement.

## Core Features

This system supports:

- Limit orders

- Market orders

- Cancel orders

- FIFO price-time priority

- Partial fills

- Deterministic replay from replay log

- Random data generation

- CLI demo runner

The implementation is in-memory, as permitted by the requirements.

## Architecture Overview

```
[Generator / Replay]
        ↓
 Order Intake Layer
        ↓
 Matching Engine
        ↓
 State Mutation
        ↓
 Event Journal
        ↓
 Logger Thread (rtrb SPSC)
```

The system is strictly layered:

### 1. Order Intake

Responsible for:

- Parsing input/Data generation

- Constructing normalized order objects

- Validating order structure

No matching or state mutation occurs here.

Data generation will leave an input log of orders that will be used for replay.

### 2. Matching Engine

The matching engine:

- Enforces price priority

- Enforces FIFO within price levels

- Supports partial fills

- Emits *BookEvents* describing all state transitions

Matching logic does not perform IO.

### 3. State Representation

The order book consists of:

- Separate bid and ask price maps

- Price levels containing:

  - head

  - tail

  - total_orders

- Orders stored in a slab (arena allocation)

- FIFO maintained via explicit linked-list pointers (prev, next)

- An order_id → slab_index map for O(1) lookup

Deletion properly updates:

- prev.next

- next.prev

- head

- tail

All state mutation occurs inside the matching engine.

### 4. Event Log/Journal

All state transitions produce immutable events:

```Rust
pub enum BookEvent {
    Match(MatchEvent),
    Cancel(CancelEvent),
    Insert(InsertEvent),
    BookSnapshot(String),
}
```

The event log acts as the single source of truth.

`BookSnapshot` represents the final state of the book after processing.

It will always be logged at the end of processing including a checksum to verify the equality of state of the order book.

## Determinism Guarantees

Determinism is enforced through:

- No randomness inside matching logic

- Stable iteration order

- Explicit FIFO linked structure

- Strict sequential event processing

- No reliance on thread scheduling for correctness

Given the same input event stream, the engine produces identical:

- Match sequences

- State transitions

- Final book state

> **NOTE:** System time is not used in any matching or business logic. It is recorded solely as a timestamp when an order enters the matching engine.

## Matching Rules

1. Price priority first

2. FIFO within price level

3. Partial fills allowed

4. Fully filled resting orders removed immediately

5. Market orders consume best price levels until filled or book is empty

## Replayability

The engine supports two modes:

### Generator Mode

Generates random input orders for testing. It will leave behind an input file of what orders it generated which can be used for replay.

```code
cargo run -- --mode gen --output events.log
```

### Replay Mode

Replays an existing event log:

```code
cargo run -- --mode replay --input events.log
```

Replay does not depend on system time or scheduling behavior.

## Concurrency Model

The matching engine itself is single-threaded to preserve determinism.

A lock-free SPSC ring buffer (rtrb::RingBuffer) is used to decouple:

- Event production (matching engine)

- Event logging (IO thread)

Termination is handled explicitly via an atomic boolean that will be flipped by the matching thread.

Concurrency does not affect matching correctness.

## Assumptions

Given the time constraints, the following simplifying assumptions were made:

- Single market only

- No funding rate logic

- No liquidation engine

- No fee calculation

- No self-trade prevention

- No advanced order types (iceberg, post-only, etc.)

- No persistence beyond event log

- Default time-in-force (GTC)

All assumptions are explicit to maintain clarity.

## Edge Cases

- Partial fill removing head

- Partial fill removing middle order

- Cancel during iteration

- Cancel of non-existent order

- Market order exceeding available liquidity

- Correct pointer updates on deletion

- Slab index reuse safety

- Clean shutdown of logging thread

## Tradeoffs

The implementation intentionally favors simplicity and determinism over feature completeness. Example tradeoffs include:

1. Price, quantity, and order id representation

    - Stored as `u64` and `u32` integers instead of floating-point or decimal types

    - Simplifies comparison, avoids floating-point rounding errors, and guarantees determinism, but requires all prices/quantities to be scaled (e.g., in micro-units) instead of being human-readable decimals

2. Single in-memory order book

    - No persistence beyond event log

    - Simplifies replay logic and testing, but the system cannot recover from process crashes

3. Single market only

    - Only one instrument is supported.

    - Makes data structures simpler (no need for a market-level map), but the engine is not directly multi-asset ready

4. Simplified order types

    - Only limit and market orders are supported.

    - No advanced order types (e.g., iceberg, post-only, stop-limit) reduces complexity and ensures deterministic FIFO behavior but limits realism

5. OrderBook data representation

    - Uses Rust's `BTreeMaps` to store price levels.

    - Simplifies implementation but not as efficient/fast as something like a fixed-size flat map indexed by price. The flat map has much better cache locality but has more complicated logic surrounding indexing and moving mid prices.

These tradeoffs are explicit to maintain clarity, determinism, and simplicity while still demonstrating correct matching behavior under replay.

## Extensibility

The design can be extended to support:

- Snapshotting for backups

- Margin accounting

- Fee model

- Advanced order types

## Submission Notes

This repository includes:

- A lightweight runnable demo (CLI)

- Deterministic matching engine

- Replay functionality

- Random data generation

- Explicit architectural separation

AI was used for:

- Refining logging concurrency patterns

- Brainstorming data structure optimizations

- Generating tests

- Validating edge-case handling

- Structuring documentation

All architectural decisions and matching logic were designed and implemented manually.

## Summary

This implementation demonstrates:

- Correct FIFO matching behavior

- Deterministic replayability

- Clean state transitions

- Clear architectural separation

- Explicit reasoning about tradeoffs and assumptions

The focus was on correctness and determinism within the expected 5–10 hour scope.
