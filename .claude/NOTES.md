# M0

## Why must both peers be able to act as sender _and_ receiver on the _same_ UDP socket?

Because both side need to start the network traffic to poke through the firewall and open a connection to the other. Incoming traffic from B is allowed if it matches any outgoing previous traffic to B. The same for A.

# M1

Run

```bash
just run --side a
```

and

```bash
just run --side b
```

starts both sides and exchanges some pings.
