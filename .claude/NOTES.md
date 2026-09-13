# M0

## Why must both peers be able to act as sender _and_ receiver on the _same_ UDP socket?

Because both side need to start the network traffic to poke through the firewall
and open a connection to the other. Incoming traffic from B is allowed if it
matches any outgoing previous traffic to B. The same for A.

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

Assumption: I assume the ip:port on side A and B, this should later be got by
STUN and then passed through UDP to the other side.

# M2

Prediction: STUN message must be send and received from a single socket
otherwise it doesnt match what the NAT has configured (SNAT). STUN only works if
the NAT's mapping is endpoint-independent. For the port Google reports to be the
same port your peer will hit, the NAT must reuse one mapping across different
destinations.

Destination NAT is configurable in the router mostly.
