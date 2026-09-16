# NAT Traversal — Assistive Learning Plan (prompt-loop format)

A guided, incremental exercise to build a UDP NAT-traversal tool in Rust that
you launch as two peers:

```bash
main --side a     # peer A
main --side b     # peer B
```

You write the code. Claude acts as a **Socratic coach in a loop**: it explains
the concept, points you at the right article section, reviews what you wrote,
and hints — it does **not** hand you the full solution unless you explicitly
ask. Claude should write simple english, not with lots of adjectives and be
precise. Answers like in paper writing.

> **Source of truth:** _How NAT traversal works_ — David Crawshaw, Tailscale.
> https://tailscale.com/blog/how-nat-traversal-works (This plan was drafted from
> working knowledge of that article, not a live fetch — keep the tab open and
> read the linked section at the start of each milestone.)

---

## How to run the loop

Each milestone is one iteration. In every iteration:

1. **Read** — the article section named in the milestone + its "Concept" note.
2. **Predict** — before coding, write 2–3 sentences in `NOTES.md` answering the
   milestone's _Learning question_. (Forces active recall.)
3. **Attempt** — implement the milestone's _Deliverable_ yourself.
4. **Loop with Claude** — say:
   `continue the NAT loop — I'm on M<n>, here's my code`. Claude will:
   - check your `NOTES.md` prediction against reality,
   - review your code for correctness + the clippy `pedantic` gate,
   - give the **smallest useful hint**, not the answer,
   - only reveal a full snippet if you say _"show me"_.
5. **Verify** — run the milestone's _Done check_.
6. **Reflect** — append one line to `NOTES.md`: what surprised you.

**Coaching contract (for Claude):** default to hints and questions. Reveal
complete implementations only on explicit request. Always tie feedback back to a
NAT concept, not just Rust syntax. Keep each iteration small enough to finish in
one sitting.

Suggested driver: `/loop continue the NAT loop` (self-paced), or just re-invoke
by hand.

---

## Concepts you will have learned by the end

- Why two hosts behind NAT can't just `connect()` to each other.
- Endpoint-independent vs endpoint-dependent **mapping** and **filtering**.
- What **STUN** does and why you need it.
- The four classic NAT behaviours: full-cone, address-restricted,
  port-restricted, **symmetric**.
- **Hole punching** / simultaneous-open and why an _outbound_ packet is what
  makes an _inbound_ one possible.
- Why you need an out-of-band **signaling channel** to swap candidate endpoints.
- The **birthday-paradox** trick for punching through symmetric NAT.
- Why production systems (Tailscale) **relay first (DERP), then upgrade** to
  direct.

---

## Milestones

### M0 — Scaffolding & roles

- **Article:** intro ("What's a NAT").
- **Concept:** the two peers are symmetric in code; `--side a`/`--side b` only
  decides who is the "connecting" side and picks distinct local ports for local
  testing.
- **Deliverable:** clean `--side {a,b}` CLI (you already have `common::Side` —
  make it a `clap::ValueEnum`), plus config for: local bind port, signaling
  address, and a place to paste the peer's endpoint. Structured logging
  (timestamps).
- **Learning question:** why must both peers be able to act as sender _and_
  receiver on the _same_ UDP socket?
- **Done check:** `main --side a` and `main --side b` both start and print their
  config.

### M1 — UDP datagrams, one socket

- **Article:** "Connectionless protocols".
- **Concept:** everything here rides on **one** `UdpSocket`. The port you send
  _from_ must be the port you receive _on_ — that single mapping is what NAT
  traversal hinges on. (No new socket per peer.)
- **Deliverable:** bind a `UdpSocket`, run a send+recv loop between the two
  local instances over `127.0.0.1`; exchange a `ping`/`pong` with sequence
  numbers.
- **Learning question:** what breaks if side A uses a fresh ephemeral socket for
  each outgoing message?
- **Done check:** A and B ping-pong locally for 10s without loss.

### M2 — STUN: discover your public `ip:port`

- **Article:** "Endpoint-independent mapping" / STUN.
- **Concept:** you cannot know your own public endpoint from inside the NAT. A
  STUN server echoes back the source `ip:port` it _saw_, which is your NAT's
  mapping.
- **Deliverable:** send a minimal **STUN Binding Request** (RFC 5389) from your
  M1 socket to a public STUN server (e.g. `stun.l.google.com:19302`) and parse
  the `XOR-MAPPED-ADDRESS` from the response. Print your reflexive endpoint.
- **Learning question:** why must the STUN request go out of the _same_ socket
  you'll use for peer data?
- **Done check:** you print a public `ip:port`;

### M3 — Signaling / rendezvous

- **Article:** "Allocating ports" → the need to exchange addresses.
- **Concept:** hole punching needs each side to know the _other's_ candidate
  endpoints. NAT can't deliver that — you need an out-of-band channel.
- **Deliverable:** simplest first: **file based** — Create two files with the
  serialized `ip:port`. Then (optional) a tiny TCP "rendezvous" server that both
  peers connect to and which swaps their candidate lists.
- **Learning question:** why can the signaling channel be slow/high-latency
  without hurting the eventual direct connection?
- **Done check:** each side ends up holding the other's `{lan, reflexive}`
  candidates.

### M4 — Hole punching (simultaneous open) — the core trick

- **Article:** "Birthday paradox"'s preamble — "Hole punching".
- **Concept:** an **outbound** packet to the peer opens a hole in _your_ NAT's
  filter so the peer's inbound packet is accepted. Both sides must fire roughly
  simultaneously and keep retrying; the first accepted packet in each direction
  "wins".
- **Deliverable:** both peers spray UDP packets at _all_ of the other's
  candidates on a timer, listen concurrently, and on first valid packet, latch
  that remote endpoint and switch to a keepalive. Handle the race where both/one
  packet is dropped.
- **Learning question:** for a port-restricted-cone NAT, exactly which prior
  outbound packet makes the peer's packet pass the filter?
- **Done check:** two peers on _different_ networks (or the M6 test harness)
  exchange data directly, no relay.

### M5 — NAT type detection

- **Article:** "Not all NATs are created equal" (the taxonomy).
- **Concept:** classify by comparing mappings/filters observed via STUN from
  multiple server IPs/ports: full-cone, address-restricted, port-restricted,
  **symmetric** (mapping changes per destination).
- **Deliverable:** probe two STUN endpoints and classify your NAT; log the
  verdict.
- **Learning question:** which single property makes symmetric NAT the hard case
  for M4?
- **Done check:** tool prints a plausible NAT class for your network.

### M6 — Symmetric NAT: port prediction & birthday paradox

- **Article:** "The birthday paradox".
- **Concept:** symmetric NAT gives a _different_ port per destination, so the
  STUN-learned port is useless for the peer. If ports are allocated ~randomly,
  have one side open many ports and the other fire at many guesses — with ~256
  each, collision probability is high (birthday math). If ports are sequential,
  predict `port+1, +2, …`.
- **Deliverable:** when M5 says symmetric, fan out: open N local sockets / fire
  M guesses, detect the hit, and collapse to the winning pair.
- **Learning question:** why does the birthday paradox make this ~`√N` cheaper
  than naive full-range scanning?
- **Done check:** a simulated symmetric NAT (see harness) is punched within your
  attempt budget more often than not.

### M7 — Relay fallback (DERP-style)

- **Article:** "When direct connections fail" / DERP.
- **Concept:** direct won't always work. Production design: connect via an
  **encrypted relay immediately** so you're never disconnected, run
  hole-punching in the background, and **upgrade** to direct when/if it succeeds
  — silently downgrading if it dies.
- **Deliverable:** a minimal relay server that forwards datagrams by peer-id;
  peers start on the relay, then switch to the direct path from M4/M6 once
  established, with fallback.
- **Learning question:** why relay-first-then-upgrade instead of
  try-direct-then-relay?
- **Done check:** kill the direct path mid-session → traffic falls back to relay
  without dropping the "connection".

### M8 — Robustness & wrap-up

- Keepalives + mapping-refresh timers (NAT mappings expire), path-change
  detection, graceful teardown, and a short `NOTES.md` retro: which NAT types
  you actually beat.

---

## Local test harness (simulating NATs without two ISPs)

Pick one, cheapest first:

1. **Loopback pseudo-NAT** (M1–M4 dev): a small in-process/UDP shim that
   rewrites source ports to emulate a mapping — fast, deterministic,
   unit-testable.
2. **Linux network namespaces + `iptables MASQUERADE`** (realistic): two `netns`
   behind NAT boxes; script `-j MASQUERADE` for cone types and `--random` for
   symmetric. Best for M5–M7. (Do this on a machine you control; needs
   root/`sudo`.)
3. **Two real machines / phone hotspot** for the honest end-to-end demo.

Add integration tests that assert "direct path established" per NAT class.

---

## Suggested Rust building blocks

- Start on **`std::net::UdpSocket`** (blocking, one thread per direction) to
  feel the mechanics; consider `tokio` only from M4 when you need concurrent
  send/listen/timers.
- STUN: hand-roll the Binding Request/`XOR-MAPPED-ADDRESS` parse (great
  learning) or read `stun_codec`/`bytecodec` for reference — but write your own
  first.
- Keep everything reusable in the `common` crate; `main` stays the thin
  CLI/orchestrator.
- Respect the repo's `clippy::pedantic = deny` gate on every milestone.

## Definition of done (whole exercise)

Two peers on genuinely different networks establish a **direct** UDP path across
at least cone-type NAT (bonus: symmetric via M6), with **relay fallback** when
direct fails — and your `NOTES.md` can explain _why each step is necessary_ in
your own words.
