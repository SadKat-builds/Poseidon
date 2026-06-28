# Learning Rust by Building a Distributed Key-Value Store

## The Core Mental Model: Think in Layers, Contracts, and Data Flow

Before writing a single line of code, internalize this framework. It applies to **every** system you'll ever build — databases, web servers, compilers, game engines, operating systems.

### The Three Pillars of Systems Thinking

```
┌─────────────────────────────────────────────────────────┐
│                    SYSTEMS THINKING                     │
├───────────────────┬──────────────────┬──────────────────┤
│     LAYERS        │    CONTRACTS     │    DATA FLOW     │
│                   │                  │                  │
│ Every system is   │ Every boundary   │ Every operation  │
│ a stack of        │ between layers   │ is data moving   │
│ abstractions,     │ has a precise    │ through a        │
│ each hiding       │ interface —      │ pipeline of      │
│ complexity from   │ what goes in,    │ transformations  │
│ the one above     │ what comes out,  │                  │
│                   │ what can fail    │                  │
└───────────────────┴──────────────────┴──────────────────┘
```

**How this applies to a key-value store:**

```
       User / Client
           │
           ▼
   ┌───────────────┐     Contract: get(key) → Option<Value>
   │   API Layer    │              put(key, value) → Result
   │  (interface)   │              delete(key) → Result
   └───────┬───────┘
           │
           ▼
   ┌───────────────┐     Contract: execute(Command) → Response
   │  Engine Layer  │     Handles: concurrency, transactions,
   │  (logic)       │              query planning
   └───────┬───────┘
           │
           ▼
   ┌───────────────┐     Contract: read(offset) → Bytes
   │ Storage Layer  │              write(bytes) → offset
   │ (persistence)  │              sync() → Result
   └───────┬───────┘
           │
           ▼
   ┌───────────────┐     Contract: syscalls (open, read, write, fsync)
   │   OS / Disk    │
   └───────────────┘
```

> [!IMPORTANT]
> **The key insight**: You don't build this top-down or bottom-up. You build it **inside-out** — start with the simplest working version of the core layer, then expand outward in both directions.

---

## The Build Plan: 7 Phases, Inside-Out

Each phase teaches specific Rust concepts **and** specific systems concepts. You'll never learn a Rust feature in isolation — it always serves a systems purpose.

---

### Phase 1: In-Memory Key-Value Store (The Core)

**What you build:** A `HashMap` wrapper with `get`, `put`, `delete` operations and a CLI REPL to interact with it.

**Systems concepts learned:**
- The **data structure** is the database (at its simplest, a DB is just a map)
- **Command parsing** — translating human intent into structured operations
- **The REPL pattern** — read, eval, print, loop (used in shells, interpreters, debuggers)

**Rust concepts learned:**
| Concept | Why You Need It Here |
|---|---|
| `HashMap<String, String>` | The core data structure |
| `enum` for commands | Model `Get`, `Put`, `Delete` as variants |
| `match` expressions | Route commands to handlers |
| `Option<T>` | A key might not exist — `get()` returns `Option` |
| `Result<T, E>` | Operations can fail — model errors explicitly |
| Ownership & borrowing | Who owns the stored values? Who can read them? |
| `String` vs `&str` | Stored data (owned) vs command input (borrowed) |
| `std::io` for REPL | Reading user input from stdin |

**The mental model takeaway:**
> Every system starts as a simple loop: **receive input → process → produce output**. The complexity comes from what happens in "process." Start there.

**Milestone:** You can type `PUT name rohit`, then `GET name` and see `rohit`.

---

### Phase 2: Persistence — Writing to Disk

**What you build:** An append-only log file. Every `PUT` and `DELETE` writes a record to a file. On startup, replay the log to rebuild the `HashMap`.

**Systems concepts learned:**
- **Write-Ahead Logging (WAL)** — the most important concept in databases
- **Crash recovery** — what happens if power dies mid-write?
- **Serialization** — turning in-memory structs into bytes on disk
- **The log as the source of truth** — the `HashMap` is just a cache of the log

**Why append-only?** Because sequential writes are **fast** (disk heads don't seek) and **safe** (you never corrupt existing data, you only add new data).

```
Log File on Disk:
┌──────────────────────────────┐
│ PUT key1 value1              │  ← record 1
│ PUT key2 value2              │  ← record 2
│ PUT key1 value1_updated      │  ← record 3 (overwrites record 1 in memory)
│ DELETE key2                  │  ← record 4
└──────────────────────────────┘

In-Memory HashMap (rebuilt on startup by replaying log):
┌──────────────────────────────┐
│ key1 → value1_updated        │
└──────────────────────────────┘
```

**Rust concepts learned:**
| Concept | Why You Need It Here |
|---|---|
| `std::fs::File`, `OpenOptions` | Opening/creating the log file |
| `BufWriter` / `BufReader` | Efficient I/O (don't syscall per byte) |
| `serde` + `serde_json` (or manual) | Serializing commands to disk |
| `impl` blocks & methods | Give your `Store` struct behavior |
| Error handling with `?` | Propagate I/O errors cleanly |
| `Drop` trait | Flush buffers when the store is dropped |
| Lifetimes (first taste) | References to data that must outlive a function |

**The mental model takeaway:**
> **Durability = writing to disk before acknowledging success.** This pattern (write-ahead logging) appears in every database (Postgres, SQLite, Redis), every message queue (Kafka), and every filesystem (journaling). If you understand WAL, you understand 80% of data systems.

**Milestone:** Kill your process, restart it, and all your data is still there.

---

### Phase 3: Compaction & Efficient Storage

**What you build:** The log grows forever — you need **compaction**. Periodically, read the entire log, keep only the latest value for each key, and write a new compacted log. Also, build an **index** (an in-memory `HashMap<Key, FileOffset>`) so you can read values directly from disk without loading everything into memory.

**Systems concepts learned:**
- **Compaction / Garbage Collection** — reclaiming space from stale data
- **Indexing** — trading memory for speed (the core tradeoff in all of CS)
- **LSM Trees vs B-Trees** — you're building the foundation of an LSM tree (like LevelDB, RocksDB, Cassandra)
- **File segments** — splitting the log into manageable chunks

```
Before Compaction:             After Compaction:
┌────────────────────┐         ┌────────────────────┐
│ PUT a 1            │         │ PUT a 3            │  ← only latest
│ PUT b 2            │   ──►   │ PUT c 4            │  ← only latest
│ PUT a 3            │         └────────────────────┘
│ PUT c 4            │         b is gone (was deleted)
│ DELETE b           │
└────────────────────┘
```

**Rust concepts learned:**
| Concept | Why You Need It Here |
|---|---|
| `struct` composition | `Store` contains `Index`, `LogFile`, `Config` |
| Generics `<K, V>` | Make your store work with any key/value types |
| `trait` definitions | Define `StorageEngine` trait — abstraction over backends |
| Iterators & closures | Process log entries functionally during compaction |
| File I/O with seeking | `Seek` trait to jump to offsets |
| `PathBuf` & `std::fs` | Managing multiple segment files |
| Modules & project structure | Split into `storage/`, `engine/`, `index/` modules |

**The mental model takeaway:**
> **Every system has a "hot path" and a "cold path."** The hot path (serving reads/writes) must be fast. The cold path (compaction, garbage collection, maintenance) runs in the background. Designing the boundary between them is a core architectural skill.

**Milestone:** Your database can handle millions of writes without running out of disk space.

---

### Phase 4: Network Layer — TCP Server

**What you build:** Replace the REPL with a TCP server. Clients connect over the network, send commands, get responses. Define a wire protocol (like Redis's RESP protocol).

**Systems concepts learned:**
- **Client-server architecture** — the foundation of all distributed systems
- **Wire protocols** — how to frame messages over a byte stream (length-prefix, delimiters, etc.)
- **Connection handling** — one thread per connection? Thread pool? Event loop?
- **Serialization/Deserialization over the wire** — same concept as disk, different medium

```
  Client A ──TCP──┐
                   │     ┌─────────────┐     ┌─────────────┐
  Client B ──TCP──┼────►│  TCP Server  │────►│   Storage   │
                   │     │  (listener)  │     │   Engine    │
  Client C ──TCP──┘     └─────────────┘     └─────────────┘
```

**Rust concepts learned:**
| Concept | Why You Need It Here |
|---|---|
| `std::net::TcpListener` | Accept connections |
| `std::net::TcpStream` | Read/write to clients |
| `std::thread::spawn` | Handle multiple clients |
| `Arc<Mutex<T>>` | Share the store across threads safely |
| `Arc<RwLock<T>>` | Multiple readers, single writer (better concurrency) |
| Rust's ownership in threading | Why Rust prevents data races at compile time |
| Custom error types | `enum StoreError { Io, Parse, NotFound, ... }` |
| `From` trait for error conversion | Convert between error types cleanly |

> [!TIP]
> This is where Rust's ownership model **clicks**. When you try to share the `Store` across threads, the compiler will **force** you to think about who owns the data, who can read it, and who can write it. This isn't a Rust quirk — it's the actual concurrency problem that causes bugs in C++/Java/Go. Rust just makes you solve it upfront.

**The mental model takeaway:**
> **The network is just another I/O layer.** The same read/parse/execute/respond loop works whether input comes from stdin, a file, or a socket. The difference is: the network is unreliable, concurrent, and adversarial. Designing for this changes everything.

**Milestone:** Open two terminals, connect both to your server, and see writes from one appear in reads from the other.

---

### Phase 5: Async I/O with Tokio

**What you build:** Replace `std::thread` with `tokio` for async I/O. Handle thousands of concurrent connections with a small thread pool.

**Systems concepts learned:**
- **Async I/O** — why threads don't scale (context switching, memory overhead)
- **Event loops** — how `epoll`/`kqueue`/`IOCP` work under the hood
- **Backpressure** — what happens when clients send faster than you can process?
- **Cooperative vs preemptive scheduling** — how async differs from threads

**Rust concepts learned:**
| Concept | Why You Need It Here |
|---|---|
| `async`/`await` syntax | Non-blocking I/O |
| `tokio::net::TcpListener` | Async network primitives |
| `tokio::sync::RwLock` | Async-aware locking |
| `Pin` and `Future` (concepts) | Why async Rust is different from async JS/Python |
| `tokio::spawn` | Spawn async tasks |
| `tokio::select!` | Wait on multiple async operations |
| Trait objects & `dyn` | Dynamic dispatch for protocol handlers |

**The mental model takeaway:**
> **Concurrency is not parallelism.** Async gives you concurrency (handling many things) without parallelism (doing many things simultaneously). Understanding this distinction is essential for building any high-performance system.

**Milestone:** Benchmark: your async server handles 10x more concurrent connections than the threaded version with the same memory.

---

### Phase 6: Replication — Multiple Nodes

**What you build:** Run multiple instances of your database. One is the **leader** (accepts writes), others are **followers** (replicate data from the leader). Clients can read from any node.

**Systems concepts learned:**
- **Replication** — copying data across machines for fault tolerance
- **Leader-follower architecture** — used by MySQL, PostgreSQL, MongoDB
- **Consistency models** — what does a reader see right after a write?
- **Replication lag** — followers are always slightly behind
- **Failure detection** — how do you know if the leader is dead?

```
                    ┌─────────────────┐
   Writes ────────► │  Leader Node    │
                    │  (read/write)   │
                    └────┬───────┬────┘
                         │       │
              Replication│       │Replication
                         ▼       ▼
                  ┌──────────┐ ┌──────────┐
   Reads ────────►│ Follower │ │ Follower │◄──── Reads
                  │  Node 1  │ │  Node 2  │
                  └──────────┘ └──────────┘
```

**Rust concepts learned:**
| Concept | Why You Need It Here |
|---|---|
| `serde` for binary serialization | Efficient replication protocol |
| `tokio::sync::broadcast` | Fan-out writes to followers |
| Channels (`mpsc`, `oneshot`) | Inter-component communication |
| `enum` for state machines | Node states: `Follower`, `Candidate`, `Leader` |
| `trait` for pluggable networking | Abstract over TCP/UDP/in-process for testing |
| Integration testing | Spin up multiple nodes in tests |
| `cfg` attributes & feature flags | Conditional compilation for testing |

**The mental model takeaway:**
> **Distribution is about managing copies.** Every distributed system problem (consistency, availability, partition tolerance — CAP theorem) reduces to: "Multiple copies of data exist. How do we keep them in sync, and what do we sacrifice when we can't?"

**Milestone:** Kill the leader, promote a follower, and no data is lost.

---

### Phase 7: Consensus — Raft Protocol

**What you build:** Implement the Raft consensus algorithm. All nodes agree on the order of operations even if some nodes crash or the network partitions.

**Systems concepts learned:**
- **Distributed consensus** — the hardest problem in distributed systems
- **Raft algorithm** — leader election, log replication, safety
- **State machines** — the same log applied to the same state machine produces the same state
- **Linearizability** — the strongest consistency guarantee

> [!NOTE]
> This phase is **hard**. Raft is the most approachable consensus algorithm, but it still has subtle edge cases. The [Raft paper](https://raft.github.io/raft.pdf) is beautifully written — read it before coding.

**Rust concepts learned:**
| Concept | Why You Need It Here |
|---|---|
| Complex `enum` state machines | Raft states with associated data |
| `tokio::time` for timeouts | Election timeouts, heartbeats |
| Property-based testing | Use `proptest` to find edge cases |
| Deterministic simulation | Test distributed protocols without real networks |
| Unsafe Rust (maybe) | Performance-critical paths |
| Advanced generics & associated types | Abstract over transport, storage, state machine |

**The mental model takeaway:**
> **Consensus is just "everyone agrees on the same log."** Once you have an agreed-upon log, you can build anything on top: a database, a lock service, a configuration store, a coordination service. This is how ZooKeeper, etcd, and CockroachDB work.

---

## The Transferable Framework

After building this project, you'll have a mental model that applies to **any** system:

### The Universal System Template

```
┌──────────────────────────────────────────────────────────────┐
│                        ANY SYSTEM                            │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  1. INTERFACE       What does the user/caller see?           │
│     └─ API, CLI, GUI, wire protocol                         │
│                                                              │
│  2. PROCESSING      What transformations happen?             │
│     └─ Parsing, validation, business logic                  │
│                                                              │
│  3. STATE           What data is held and how?               │
│     └─ In-memory structures, persistence format             │
│                                                              │
│  4. CONCURRENCY     How are multiple users handled?          │
│     └─ Threads, async, locks, message passing               │
│                                                              │
│  5. DURABILITY      How is data kept safe?                   │
│     └─ Write-ahead logs, checkpoints, replication           │
│                                                              │
│  6. DISTRIBUTION    How does it scale beyond one machine?    │
│     └─ Sharding, replication, consensus                     │
│                                                              │
│  7. OBSERVABILITY   How do you know it's working?            │
│     └─ Logging, metrics, tracing                            │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

Every system you encounter — Redis, Kafka, PostgreSQL, Linux kernel, a game engine, a compiler — maps onto this template. The details change; the structure doesn't.

### How to Apply This to Other Systems

| System | Interface | State | Durability | Distribution |
|---|---|---|---|---|
| **Web Server** | HTTP | Request context | Access logs | Load balancer |
| **Message Queue** | Pub/Sub API | In-flight messages | WAL + replication | Partitioning |
| **File System** | POSIX API | Inodes + blocks | Journaling | Network FS (NFS) |
| **Compiler** | Source code | AST + IR | Build cache | Distributed build |
| **Game Engine** | Game loop | ECS world state | Save files | Netcode |

---

## How to Actually Work Through This

### The Rule of "Make It Work, Make It Right, Make It Fast"

For **each phase**:

1. **Sketch the types first.** Write the `struct`s and `enum`s. Define the `trait`s. Don't implement anything yet. This is your contract.

2. **Write failing tests.** What should `get` return for a non-existent key? What happens after a `put` then `delete`? Write these as `#[test]` functions.

3. **Make the tests pass.** The simplest, ugliest implementation that works. Ignore performance. Ignore edge cases. Just make the green check appear.

4. **Refactor.** Now make it clean. Extract modules. Add proper error types. This is where you learn Rust's type system deeply.

5. **Benchmark & stress-test.** Only now think about performance. Use `criterion` for benchmarks. Throw a million operations at it. Find the bottleneck.

### Reading Recommendations Per Phase

| Phase | Read This |
|---|---|
| 1 | [The Rust Book](https://doc.rust-lang.org/book/) chapters 1-8 |
| 2 | [Designing Data-Intensive Applications](https://dataintensive.net/) chapter 3 (Storage & Retrieval) |
| 3 | [The Rust Book](https://doc.rust-lang.org/book/) chapters 10-11 (Generics, Traits, Testing) |
| 4 | [Designing Data-Intensive Applications](https://dataintensive.net/) chapter 1-2 + [Rust Book](https://doc.rust-lang.org/book/) chapter 16 (Concurrency) |
| 5 | [Tokio tutorial](https://tokio.rs/tokio/tutorial) |
| 6 | [DDIA](https://dataintensive.net/) chapter 5 (Replication) |
| 7 | [The Raft Paper](https://raft.github.io/raft.pdf) |

### The Project Structure You'll Grow Into

```
distributed-key-value-database/
├── Cargo.toml
├── src/
│   ├── main.rs              ← Entry point, CLI/server startup
│   ├── lib.rs               ← Re-exports, top-level API
│   ├── store/
│   │   ├── mod.rs           ← Store trait + in-memory impl
│   │   ├── memory.rs        ← Phase 1: HashMap store
│   │   └── log.rs           ← Phase 2: Append-only log store
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── segment.rs       ← Phase 3: Log segments
│   │   ├── index.rs         ← Phase 3: Offset index
│   │   └── compaction.rs    ← Phase 3: Compaction logic
│   ├── network/
│   │   ├── mod.rs
│   │   ├── protocol.rs      ← Phase 4: Wire protocol
│   │   ├── server.rs        ← Phase 4/5: TCP server
│   │   └── client.rs        ← Phase 4/5: Client library
│   ├── replication/
│   │   ├── mod.rs
│   │   ├── leader.rs        ← Phase 6: Leader logic
│   │   └── follower.rs      ← Phase 6: Follower logic
│   └── consensus/
│       ├── mod.rs
│       ├── raft.rs           ← Phase 7: Raft implementation
│       ├── log.rs            ← Phase 7: Raft log
│       └── state_machine.rs  ← Phase 7: Applied state machine
├── tests/
│   ├── integration_tests.rs
│   └── simulation.rs
└── benches/
    └── throughput.rs
```

> [!TIP]
> **You won't build this structure on day one.** It emerges naturally as you add phases. In Phase 1, you'll have just `main.rs` with ~100 lines. That's perfect. Let the architecture grow from the code, not the other way around.

---

## The One Question That Drives Everything

At every decision point, ask yourself:

> **"What happens when this fails?"**

- What happens when the disk is full?
- What happens when the network drops?
- What happens when two clients write the same key at the same time?
- What happens when the process crashes mid-write?
- What happens when a node goes down?

This question is what separates toy projects from real systems. Rust's type system (especially `Result` and `Option`) forces you to answer it at compile time. That's the superpower.

---

## Getting Started: Your First 30 Minutes

Start Phase 1 right now. Here's your first task:

1. Define a `Command` enum with variants: `Get { key: String }`, `Put { key: String, value: String }`, `Delete { key: String }`
2. Define a `Store` struct wrapping a `HashMap<String, String>`
3. Implement `execute(&mut self, cmd: Command) -> Result<Option<String>, String>` on `Store`
4. Build a REPL in `main()` that reads lines from stdin, parses them into `Command`s, and executes them

When you're ready to start coding any phase, just tell me which one and I'll guide you through it step by step.
