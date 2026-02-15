# CRDT Sync, E2E Encryption & Infrastructure Research

**Date**: 2026-02-15
**Purpose**: Research for Sakya sync server architecture: CRDT selection, E2E encryption patterns, deployment tooling, and device pairing.

---

## Table of Contents

1. [Spell's .ancillary CRDT Server](#1-spells-ancillary-crdt-server)
2. [Modern Rust CRDT Libraries (2025-2026)](#2-modern-rust-crdt-libraries-2025-2026)
3. [E2E Encrypted CRDT Syncing Patterns](#3-e2e-encrypted-crdt-syncing-patterns)
4. [deploy-rs (Nix Deployment Tool)](#4-deploy-rs-nix-deployment-tool)
5. [QR Code Authentication / Device Pairing Patterns](#5-qr-code-authentication--device-pairing-patterns)

---

## 1. Spell's .ancillary CRDT Server

### Research Status: INCONCLUSIVE

Despite extensive searching with multiple query variations ("Spell app CRDT", "Spell ancillary CRDT server", "Spell text editor CRDT sync", "Spell writing app ancillary server", "Spell app by Silvio Rizzi"), **no product called "Spell" with a CRDT-based sync architecture was found** in public sources.

What was found:
- **Spell Catcher X**: An old macOS spell-checking utility (OS X 10.5-10.7 era), unrelated to CRDTs.
- **Silvio Rizzi** (developer of Reeder, Mela): No writing editor called "Spell" found in his portfolio.
- No Hacker News discussions, blog posts, or GitHub repositories referencing a "Spell" writing app with CRDT syncing.

**Possible explanations**: The product may be unreleased/private, use a different public name, or the name may be slightly different from what was searched.

**Recommendation**: If this refers to a specific product you have seen, please provide a URL, developer name, or additional context so the search can be refined.

---

## 2. Modern Rust CRDT Libraries (2025-2026)

### 2.1 Loro

| Attribute | Details |
|---|---|
| **Repository** | [github.com/loro-dev/loro](https://github.com/loro-dev/loro) |
| **Docs** | [loro.dev](https://loro.dev/), [docs.rs/loro](https://docs.rs/loro/) |
| **Language** | Rust core, bindings for JS (WASM), Swift |
| **Maturity** | Active development, approaching v1.0, rapidly evolving |
| **Algorithm** | Replayable Event Graph (REG) |

**Features**:
- **Data types**: Text (rich text via Peritext + Fugue), List, Map (LWW), Movable Tree, Movable List
- **Rich text**: First-class support via `LoroText` with `insert`, `delete`, `apply_delta`, `mark` (bold, italic, links, custom styles), stable cursor positions
- **History**: Built-in complete editing history, time travel, version tracking without disabling GC
- **Movable trees**: Native move operations for hierarchical data (critical for folder/chapter structures)

**Binary Format & Encoding**:
- Header: 4-byte magic number, 16-byte checksum, 2-byte encode mode
- Export modes via `doc.export(mode)`: `snapshot`, `update`, `updates-in-range`
- All formats importable via `doc.import(bytes)`

**Network Protocol (Loro Protocol)**:
- Wire protocol for real-time CRDT sync over WebSocket
- Multiplexes multiple independent "rooms" over a single WebSocket connection
- Magic byte prefixes: `%LOR` (Loro Doc), `%EPH` (ephemeral/cursors), `%ELO` (E2E encrypted Loro), `%YJS`/`%YAW` (Yjs interop)
- Large updates split into fragments under 256 KiB for WebSocket platform limits
- Application-level keepalive (ping/pong text frames)
- Rust client (`loro-websocket-client`) and server implementations
- **Source**: [loro.dev/blog/loro-protocol](https://loro.dev/blog/loro-protocol)

**Performance**:
- Focuses on memory usage, CPU performance, and loading speed
- Snapshot mode significantly reduces loading time (contains ops + in-memory representation)
- Competitive with Yjs in most benchmarks (same order of magnitude)
- WASM bundle >1MB (plus ~30% base64 encoding overhead if inlined)

**E2E Encryption Compatibility**:
- `%ELO` message type in the wire protocol explicitly supports end-to-end encrypted Loro documents
- Architecture separates encrypted documents from ephemeral data

**Verdict**: Best choice for Sakya given rich text support, movable trees (chapter organization), built-in history (time travel for manuscript versions), Rust-native with WASM bindings, and explicit E2E encryption support in the wire protocol.

---

### 2.2 Yrs (y-crdt) -- Rust Port of Yjs

| Attribute | Details |
|---|---|
| **Repository** | [github.com/y-crdt/y-crdt](https://github.com/y-crdt/y-crdt) |
| **Docs** | [docs.rs/yrs](https://docs.rs/yrs) |
| **Language** | Rust, with C FFI, WASM, Python, Ruby, .NET, Swift bindings |
| **Maturity** | Mature, production-ready, NLnet-funded |
| **Algorithm** | YATA (Yet Another Transformation Approach) |

**Features**:
- Compatible with Yjs binary format (v1 and v2)
- State vectors, delta diff updates, merges
- Subscription events for incoming updates
- Shared types: Text, Array, Map, XML Fragment, XML Element, XML Text

**Binary Format**:
- lib0 encoding with two variants compatible with Yjs
- Highly compact, optimized for small payload size and high encoding/decoding speed

**Performance**:
- Potential for better performance than Yjs due to Rust's manual memory management
- Novel internal data structures for efficient CRDT representation

**E2E Encryption Compatibility**:
- No built-in E2E encryption support
- Used by Secsync as the CRDT backend (proving it works with E2E encryption layers)
- Used by Bolik Timeline (Roman Zaynetdinov's E2EE local-first app)

**Limitations**:
- No built-in movable tree support
- No native history/time-travel
- Rich text support through Yjs ecosystem (not as advanced as Loro's Peritext implementation)

**Source**: [bartoszsypytkowski.com/yrs-architecture](https://www.bartoszsypytkowski.com/yrs-architecture/), [nlnet.nl/project/Yrs](https://nlnet.nl/project/Yrs/)

---

### 2.3 Automerge

| Attribute | Details |
|---|---|
| **Repository** | [github.com/automerge/automerge](https://github.com/automerge/automerge) |
| **Docs** | [automerge.org](https://automerge.org/) |
| **Language** | Rust core, JS/WASM, Python, C bindings |
| **Maturity** | Production-ready (v2.0+), Automerge 3 released |
| **Algorithm** | RGA-based with optimizations |

**Features**:
- JSON-like data model: nested maps, arrays, text, counters
- Complete document history with change tracking
- Built-in sync protocol (based on [arxiv.org/abs/2012.00472](https://arxiv.org/abs/2012.00472))
- Transport-agnostic (TCP, WebSocket, WebRTC)

**Performance (v2.0)**:
- ~260k text editing operations: 1,816ms (down from 13,052ms in v1.0)
- Memory: 44.5MB (down from 184MB in v1.0)
- Binary format: ~30% overhead vs plain text (~1 extra byte per character)
- Automerge 3: ~10x reduction in memory usage

**Sync Protocol**:
- CBOR-encoded messages (V1 and V2 formats)
- V2 supports compressed document format
- WebSocket wire protocol: handshake (peer IDs + protocol version) then sync loop
- Ephemeral messages for presence/cursors
- Single round-trip to bring two clients up to date
- **Source**: [automerge.org/automerge/automerge/sync](https://automerge.org/automerge/automerge/sync/index.html)

**E2E Encryption Compatibility**:
- Used by Ink & Switch's Keyhive project (compress-then-encrypt approach)
- Binary format enables range-based encryption of changes

**Limitations**:
- Rich text via Peritext (still evolving, not as mature as Loro's implementation)
- Higher parseTime when initial document is not empty (no snapshot optimization)
- Larger WASM bundle than pure JS Yjs

---

### 2.4 Diamond Types

| Attribute | Details |
|---|---|
| **Repository** | [github.com/josephg/diamond-types](https://github.com/josephg/diamond-types) |
| **Docs** | [docs.rs/diamond-types](https://docs.rs/diamond-types) |
| **Language** | Rust, npm packages (Node.js + browser via WASM) |
| **Maturity** | WIP / experimental, Cargo package "quite out of date" |
| **Author** | Joseph Gentle (former Google Wave engineer) |

**Features**:
- **Plain text only** (JSON-style types on `more_types` branch)
- Unique character IDs: `(client_id, sequence_number)` tuples
- Interoperable with positional updates (OT-style API)
- Uses range tree (modified B-tree) internally instead of linked list

**Performance**:
- Claims "the world's fastest CRDT"
- 5000x faster than competing implementations (then additional 10-80x improvement)
- Almost identical to Yjs conceptually, but different internal data structure

**Limitations**:
- **Plain text only** -- no rich text, no maps, no trees
- WIP status, API unstable
- Not suitable for production use in complex document applications

**Source**: [josephg.com/blog/crdts-go-brrr](https://josephg.com/blog/crdts-go-brrr/)

---

### 2.5 Comparison Matrix

| Feature | Loro | Yrs | Automerge | Diamond Types |
|---|---|---|---|---|
| **Rich text** | Peritext + Fugue | Basic (via Yjs) | Peritext (evolving) | None |
| **Movable tree** | Yes | No | No | No |
| **JSON-like data** | Yes | Yes | Yes | No (plain text) |
| **History/time-travel** | Built-in | No | Yes | No |
| **Sync protocol** | Loro Protocol | Yjs protocol | Automerge sync | None |
| **E2E encryption** | `%ELO` in protocol | Via Secsync | Via Keyhive | N/A |
| **Binary format** | Custom, compact | lib0 (Yjs compat) | CBOR-based | Custom |
| **Rust-native** | Yes | Yes | Yes | Yes |
| **WASM bundle** | >1MB | Small | Medium | Small |
| **Maturity** | Pre-1.0 | Mature | Production | WIP |
| **GC** | Planned | Yes | Planned | N/A |

---

## 3. E2E Encrypted CRDT Syncing Patterns

### 3.1 The Core Challenge

CRDTs need to merge operations from multiple peers. E2E encryption means the server cannot read or process those operations. The fundamental tension: **how do you merge data the server cannot see?**

**Answer**: The server becomes a **dumb relay**. All merging happens client-side. The server stores encrypted blobs and relays them to authorized peers.

### 3.2 Secsync (by Nik Graf)

**The most complete, practical architecture for E2EE CRDTs available today.**

| Attribute | Details |
|---|---|
| **Repository** | [github.com/nikgraf/secsync](https://github.com/nikgraf/secsync) |
| **Website** | [secsync.com](https://www.secsync.com/) |
| **Funding** | NLnet |
| **CRDT Backend** | Yjs (reference implementation) |

**Architecture**:

A Document consists of:
- **Snapshots**: Encrypted CRDT document state at a point in time
- **Updates**: Encrypted CRDT delta operations referencing a snapshot
- **Ephemeral messages**: Encrypted transient data (cursors, presence)

**Encryption**:
- **AEAD**: XChaCha20-Poly1305-IETF for all messages
- **Signing**: Ed25519 signatures on BLAKE2b hashes of `(ciphertext, nonce, public_data)`
- **Commitment verification**: `HMAC(nonce + ciphertext + AAD, key)` verified before decryption
- Public data always contains the document ID (for message-document binding)

**Hash Chain (Integrity)**:
- Parent snapshot proof: `hash(parent_proof, ciphertext_hash)`
- Creates a verifiable chain across snapshots even with intermediate gaps
- `parentSnapshotUpdateClocks`: maps client public keys to clock values (verifies snapshot creator received prior updates)

**Update Anchoring**:
- Each update includes a counter in public data, anchored to a specific snapshot
- Prevents updates from being replayed against wrong snapshots

**Ephemeral Replay Prevention**:
- Internal counter (encrypted) per session
- References document ID, snapshot ID, and optionally source update ID

**Key Management**:
- Key exchange/rotation is **out of scope** for the protocol
- Can be layered with Signal Protocol, MLS, or PKI-based lockboxes

**Source**: [secsync.com/docs/specification](https://www.secsync.com/docs/specification)

---

### 3.3 Keyhive (by Ink & Switch)

**The most ambitious E2EE CRDT project, integrating access control with encryption.**

| Attribute | Details |
|---|---|
| **Project** | [inkandswitch.com/project/keyhive](https://www.inkandswitch.com/project/keyhive/) |
| **Notebook** | [inkandswitch.com/keyhive/notebook](https://www.inkandswitch.com/keyhive/notebook/) |
| **CRDT Backend** | Automerge |

**Three-Layer Architecture**:

1. **Convergent Capabilities**: Capability-based access control model for CRDTs
   - Roles: Read, Write, Admin, Pull (retrieve bytes but not decrypt)
   - Chains of signed delegations for distributed authorization
   - Offline-verifiable without central server

2. **Group Management CRDT**: Self-certifying, concurrent group management
   - Coordination-free revocation
   - Resolves concurrent membership changes

3. **E2EE with Causal Keys**:
   - Post-compromise security
   - Symmetric key management granting access to causal predecessors
   - **BeeKEM**: Novel concurrency-friendly Continuous Group Key Agreement (CGKA)
   - Can be updated concurrently with arbitrary divergence between replicas

**Encryption Strategy (compress-then-encrypt)**:
- Does NOT encrypt individual Automerge operations (would prevent compression, bloating documents)
- Uses Automerge Binary Format to **compress ranges of changes**, then encrypts the compressed block
- Balances security with storage efficiency

**Sync Protocol (Beelay)**:
- Ed25519 authenticated messages with audience binding and timestamp validation
- Sync layers: Membership graph -> Document collection state -> Per-document content
- **RIBLT** (Rateless Invertible Bloom Lookup Tables) for efficient set reconciliation
  - 1 billion items with 5 differences = ~240 bytes
- **Sedimentree**: Hierarchical compression for older commits (progressively larger blocks)
- Common case: 2 round trips for sync

**Source**: [inkandswitch.com/keyhive/notebook/05](https://www.inkandswitch.com/keyhive/notebook/05/)

---

### 3.4 Bolik Timeline Approach (Roman Zaynetdinov)

**Practical implementation of E2EE local-first with Yrs.**

| Attribute | Details |
|---|---|
| **Blog Post** | [zaynetro.com/post/how-to-build-e2ee-local-first-app](https://www.zaynetro.com/post/how-to-build-e2ee-local-first-app) |
| **Repository** | [github.com/zaynetro/timeline](https://github.com/zaynetro/timeline) |
| **Stack** | Rust + Flutter, Yrs, OpenMLS, SQLite + LiteFS |

**Architecture Decisions**:
- **MLS** (Messaging Layer Security) for secrets sharing and device grouping only
- Application data encrypted with **ChaCha20Poly1305** and uploaded separately (not through MLS)
- One MLS group per contact, document secrets shared per-contact
- **Signature chain** (Git-like block chain) to order MLS commits (solves MLS's total-order requirement)
- Complete documents uploaded on changes (not deltas) due to encryption constraints
- Server cannot merge -> clients fetch and merge locally
- **Vector clocks** for device-specific counters and reconciliation

**Key Tradeoffs**:
- Sacrifices some MLS security guarantees to enable historical document access for new devices
- Requires existing device to log into account (no key recovery/backup yet)
- Rich text still evolving (uses Fleather/Quill delta format)

---

### 3.5 Homomorphic Encryption Approach

**Theoretical but impractical for real applications.**

- Explored by Jake Lazaroff: [jakelazaroff.com/words/homomorphically-encrypted-crdts/](https://jakelazaroff.com/words/homomorphically-encrypted-crdts/)
- Allows merge operations on encrypted data without decryption
- **Performance**: ~2 billion times slower than plaintext operations
- **Key size**: 32-byte register required 123 MB server key (~850,000x more key than data)
- **Constraint**: Code must execute worst-case branches (no conditional optimization)
- **Verdict**: Interesting academically, completely impractical for production use

---

### 3.6 Academic Work

- **Martin Kleppmann** (TU Munich): Research on adapting secure group messaging protocols for encrypted CRDTs. Proposes sending CRDT operations via secure messaging protocols to inherit their security properties. Key challenge: properties that hold for 2-party communication don't easily generalize to groups.
  - [martin.kleppmann.com/2019/05/15/encrypted-crdts.html](https://martin.kleppmann.com/2019/05/15/encrypted-crdts.html)
- **Weidner, Kleppmann et al.**: "Key Agreement for Decentralized Secure Group Messaging with Strong Security Guarantees" (ACM CCS 2021)
  - [eprint.iacr.org/2020/1281.pdf](https://eprint.iacr.org/2020/1281.pdf)
- **Secure CRDTs** (2020): Formalization of security for CRDTs using fully homomorphic encryption
  - [eprint.iacr.org/2020/944.pdf](https://eprint.iacr.org/2020/944.pdf)

---

### 3.7 Practical Pattern Summary for Sakya

The **established, practical pattern** (used by Secsync, Bolik Timeline, and partially Keyhive) is:

1. **Encrypt-then-relay**: Encrypt CRDT operations/snapshots client-side before sending to server
2. **Server = dumb relay**: Server stores and forwards encrypted blobs, cannot read or merge them
3. **Client-side merge**: Clients download encrypted updates, decrypt locally, merge via CRDT
4. **Symmetric encryption**: Use AEAD (XChaCha20-Poly1305 or ChaCha20-Poly1305) for data encryption
5. **Asymmetric signing**: Ed25519 for authentication and integrity
6. **Key management**: Separate concern -- use MLS, Signal Protocol, or custom PKI
7. **Snapshots + deltas**: Periodic snapshots (encrypted full state) + delta updates to reduce load time
8. **Hash chains**: Link snapshots/updates cryptographically for integrity verification

---

## 4. deploy-rs (Nix Deployment Tool)

### Overview

| Attribute | Details |
|---|---|
| **Repository** | [github.com/serokell/deploy-rs](https://github.com/serokell/deploy-rs) |
| **Blog Post** | [serokell.io/blog/deploy-rs](https://serokell.io/blog/deploy-rs) |
| **Language** | Rust |
| **Maintainer** | Serokell |
| **License** | MPL-2.0 |

deploy-rs is a **stateless, flake-based Nix deployment tool** written in Rust. It deploys Nix profiles to remote machines via SSH, with multi-profile support and automatic rollback safety.

### Architecture

**Local components** (deploying machine):
- `deploy` CLI: Parses args, orchestrates the workflow
- Profile Builder: Compiles Nix derivations (locally or remotely)
- Profile Pusher: Transfers built artifacts via SSH (`nix copy`)

**Remote components** (target nodes):
- `activate-rs` binary with three modes: `activate`, `wait`, `revoke`

### Deployment Flow

1. **Build**: `nix build` locally or on target
2. **Push**: `nix copy` closure to target (skipped if remote-built)
3. **Activate**: Execute activation scripts with safety monitoring

### Configuration (flake.nix)

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    deploy-rs.url = "github:serokell/deploy-rs";
  };

  outputs = { self, nixpkgs, deploy-rs }: {
    # Define what to deploy
    deploy.nodes.my-server = {
      hostname = "192.168.1.100";
      sshUser = "deploy";
      profiles = {
        # NixOS system profile (as root)
        system = {
          user = "root";
          path = deploy-rs.lib.x86_64-linux.activate.nixos
            self.nixosConfigurations.my-server;
        };
        # Custom Rust binary (as unprivileged user)
        sakya-sync = {
          user = "sakya";
          path = deploy-rs.lib.x86_64-linux.activate.custom
            self.packages.x86_64-linux.sakya-sync
            "./bin/activate";
        };
      };
    };

    # Validation checks
    checks = builtins.mapAttrs
      (system: deployLib: deployLib.deployChecks self.deploy)
      deploy-rs.lib;
  };
}
```

### Key Concepts

**Nodes**: Target machines with hostname + SSH config
**Profiles**: Independent deployable units within a node
**Activation types**:
- `activate.nixos` -- NixOS system activation
- `activate.darwin` -- macOS via nix-darwin
- `activate.home-manager` -- User-level configs
- `activate.custom <package> <script>` -- Arbitrary binaries
- `activate.noop` -- Install only, no activation

**Configuration cascade**: Deploy-level -> Node-level -> Profile-level (later overrides earlier)

### Magic Rollback (Safety Feature)

The signature safety mechanism:
1. Activation runs in background
2. A lock file is created in temp directory
3. Deployer must confirm by deleting the lock file within timeout (default 30s)
4. If not confirmed (network loss, broken SSH, bad config), **automatic rollback** to previous profile
5. Protects against: firewall lockouts, SSH key changes, network config mistakes

### Comparison to Alternatives

| Tool | Stateful | Multi-Profile | Flakes | Rollback | Non-NixOS |
|---|---|---|---|---|---|
| **deploy-rs** | No | Yes | Yes | Magic rollback | Yes |
| **Colmena** | No | Limited | Yes | No | No |
| **NixOps** | Yes | No | Partial | Manual | Limited |
| **morph** | No | No | No | No | No |

### Deploying a Rust Sync Server

For Sakya's use case (deploying a Rust binary to a NixOS server):

1. Define the Rust package in `flake.nix` outputs
2. Create an activation script that:
   - Stops the existing systemd service
   - Copies the binary to the right location
   - Starts the service
3. Use `activate.custom` with the package and activation script
4. Deploy with `deploy .#my-server.sakya-sync`

Magic rollback ensures that if the new binary crashes or breaks networking, the previous version is automatically restored.

**Source**: [deepwiki.com/serokell/deploy-rs](https://deepwiki.com/serokell/deploy-rs)

---

## 5. QR Code Authentication / Device Pairing Patterns

### 5.1 Signal Protocol (Device Linking)

**Source**: [signal.org/blog/a-synchronized-start-for-linked-devices](https://signal.org/blog/a-synchronized-start-for-linked-devices/)

**QR Code Contents**:
1. A temporary **provisioning address** (registered with Signal servers)
2. **Public key** for a locally-generated Curve25519 keypair

**Linking Flow**:
1. New device generates ephemeral Curve25519 keypair
2. New device registers provisioning address with Signal servers
3. New device displays QR code containing provisioning address + public key
4. Primary device scans QR code
5. Primary device sends **provisioning message** (encrypted with new device's public key) to the provisioning address
6. Provisioning message contains:
   - Shared account keys (identity key pair)
   - Account information
   - One-time-use linking token (cryptographic proof of permission)
   - One-time 256-bit AES key for message history archive
7. Linked devices share identity key pair but get **independent prekeys** (independent session keys)

**Message History Transfer**:
- Primary compresses message history into E2E encrypted archive
- Encrypted with the one-time AES key from provisioning message
- Media from past 45 days referenced (not re-uploaded)

**Key Property**: Linked devices share identity but have independent sessions.

---

### 5.2 WhatsApp Multi-Device Protocol

**Source**: [engineering.fb.com/2021/07/14/security/whatsapp-multi-device](https://engineering.fb.com/2021/07/14/security/whatsapp-multi-device/), [infoq.com/news/2021/07/WhatsApp-signal-protocol](https://www.infoq.com/news/2021/07/WhatsApp-signal-protocol/)

**Key Architecture Differences from Signal**:
- Each device has its **own identity key** (not shared like Signal)
- Server maintains mapping: account -> list of device identities
- Uses **client-fanout**: sender encrypts message N times for N recipient devices
- Each message encrypted with pairwise sessions per device

**QR Code Pairing**:
1. QR code contains: companion device's identity key + ephemeral linking secret key
2. Linking secret never sent to server
3. Primary scans QR, saves companion identity key
4. Primary creates **Account Signature** (signs new device's public Identity Key)
5. Companion creates **Device Signature** (signs primary's public Identity Key)
6. Both signatures exchanged -> E2E encrypted sessions established
7. Biometric authentication required on primary device (where enabled)

**Security Codes**: Extended to represent combination of ALL device identities for verification.
**Automatic Device Verification**: Devices establish mutual trust without repeated manual comparison.

---

### 5.3 Matrix Protocol (Cross-Signing + QR Verification)

**Source**: [github.com/uhoreg/matrix-doc/blob/qr_key_verification/proposals/1543-qr_code_key_verification.md](https://github.com/uhoreg/matrix-doc/blob/qr_key_verification/proposals/1543-qr_code_key_verification.md), [matrix.org/blog/2020/05/06/cross-signing-and-end-to-end-encryption-by-default-is-here](https://matrix.org/blog/2020/05/06/cross-signing-and-end-to-end-encryption-by-default-is-here/)

**Key Hierarchy**:
- **Master key**: Signs self-signing key and user-signing key
- **Self-signing key**: Signs user's own device keys
- **User-signing key**: Signs other users' master keys

**QR Code Format** (binary):
- "MATRIX" identifier
- Version and mode information
- Event ID
- User's cross-signing key
- Other user's cross-signing key
- Shared secret

**Verification Flow**:
1. Both devices display QR codes
2. Either device scans the other's QR code
3. Scanner's device notifies scanned device
4. Both verify the shared secret matches
5. Cross-signing keys are signed as trusted

**Key Advantage**: Verify once, trust everywhere. Users verify people, not individual devices.

---

### 5.4 Recommended Pattern for Sakya

Based on Signal's approach (simplest and most secure for single-user multi-device):

**Phase 1: Device Linking**
1. New device generates ephemeral X25519 keypair
2. New device displays QR code containing: `{device_id, public_key, server_url}`
3. Primary device scans QR code
4. Primary encrypts provisioning payload with new device's public key:
   - Account identity keys
   - Document encryption keys (or key derivation material)
   - Server authentication credentials
   - One-time linking token
5. Primary sends encrypted payload to server, addressed to new device's `device_id`
6. New device fetches and decrypts provisioning payload
7. Both devices now share document encryption keys

**Phase 2: Ongoing Sync**
- Use Loro's `%ELO` (E2E encrypted) message type
- CRDT operations encrypted with shared document key before transmission
- Server relays encrypted blobs between authenticated devices
- Key rotation handled via secure device-to-device channel

**Phase 3: Device Deauthorization**
- Primary device removes target from authorized device list
- Key rotation for all documents (forward secrecy)
- Server stops relaying to deauthorized device

---

## References (All Sources)

### CRDT Libraries
- [github.com/loro-dev/loro](https://github.com/loro-dev/loro)
- [loro.dev/blog/loro-protocol](https://loro.dev/blog/loro-protocol)
- [loro.dev/blog/crdt-richtext](https://loro.dev/blog/crdt-richtext)
- [loro.dev/docs/performance](https://loro.dev/docs/performance)
- [github.com/y-crdt/y-crdt](https://github.com/y-crdt/y-crdt)
- [docs.rs/yrs](https://docs.rs/yrs)
- [bartoszsypytkowski.com/yrs-architecture](https://www.bartoszsypytkowski.com/yrs-architecture/)
- [github.com/automerge/automerge](https://github.com/automerge/automerge)
- [automerge.org/blog/automerge-2](https://automerge.org/blog/automerge-2/)
- [github.com/josephg/diamond-types](https://github.com/josephg/diamond-types)
- [josephg.com/blog/crdts-go-brrr](https://josephg.com/blog/crdts-go-brrr/)
- [discuss.yjs.dev/t/yjs-vs-loro-new-crdt-lib/2567](https://discuss.yjs.dev/t/yjs-vs-loro-new-crdt-lib/2567)
- [github.com/dmonad/crdt-benchmarks](https://github.com/dmonad/crdt-benchmarks)

### E2E Encryption + CRDTs
- [github.com/nikgraf/secsync](https://github.com/nikgraf/secsync)
- [secsync.com/docs/specification](https://www.secsync.com/docs/specification)
- [inkandswitch.com/project/keyhive](https://www.inkandswitch.com/project/keyhive/)
- [inkandswitch.com/keyhive/notebook/05](https://www.inkandswitch.com/keyhive/notebook/05/)
- [zaynetro.com/post/how-to-build-e2ee-local-first-app](https://www.zaynetro.com/post/how-to-build-e2ee-local-first-app)
- [jakelazaroff.com/words/homomorphically-encrypted-crdts](https://jakelazaroff.com/words/homomorphically-encrypted-crdts/)
- [martin.kleppmann.com/2019/05/15/encrypted-crdts.html](https://martin.kleppmann.com/2019/05/15/encrypted-crdts.html)
- [eprint.iacr.org/2020/1281.pdf](https://eprint.iacr.org/2020/1281.pdf)
- [eprint.iacr.org/2020/944.pdf](https://eprint.iacr.org/2020/944.pdf)

### Deployment
- [github.com/serokell/deploy-rs](https://github.com/serokell/deploy-rs)
- [serokell.io/blog/deploy-rs](https://serokell.io/blog/deploy-rs)
- [deepwiki.com/serokell/deploy-rs](https://deepwiki.com/serokell/deploy-rs)
- [discourse.nixos.org/t/deployment-tools-evaluating-nixops-deploy-rs-and-vanilla-nix-rebuild/36388](https://discourse.nixos.org/t/deployment-tools-evaluating-nixops-deploy-rs-and-vanilla-nix-rebuild/36388)

### QR Code / Device Pairing
- [signal.org/blog/a-synchronized-start-for-linked-devices](https://signal.org/blog/a-synchronized-start-for-linked-devices/)
- [engineering.fb.com/2021/07/14/security/whatsapp-multi-device](https://engineering.fb.com/2021/07/14/security/whatsapp-multi-device/)
- [infoq.com/news/2021/07/WhatsApp-signal-protocol](https://www.infoq.com/news/2021/07/WhatsApp-signal-protocol/)
- [github.com/uhoreg/matrix-doc/.../1543-qr_code_key_verification.md](https://github.com/uhoreg/matrix-doc/blob/qr_key_verification/proposals/1543-qr_code_key_verification.md)
- [matrix.org/blog/2020/05/06/cross-signing-and-end-to-end-encryption-by-default-is-here](https://matrix.org/blog/2020/05/06/cross-signing-and-end-to-end-encryption-by-default-is-here/)

### General CRDT Resources
- [crdt.tech/implementations](https://crdt.tech/implementations)
- [mattweidner.com/2025/05/21/text-without-crdts.html](https://mattweidner.com/2025/05/21/text-without-crdts.html)
- [inkandswitch.com/peritext](https://www.inkandswitch.com/peritext/)
