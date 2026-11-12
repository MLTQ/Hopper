# peer_address.rs

An endpoint derives its preferred UDP port from its persistent public identity and binds the
same port after restart. IPv6 is optional. If the preferred port is occupied, Iroh uses its
ordinary dynamic bindings. When dialing a paired device, keep the advertised hints and relay
and try its preferred port on known private LAN addresses. Iroh authenticates the endpoint ID
on every route. A regression test pairs with an old LAN hint whose original port is closed.
