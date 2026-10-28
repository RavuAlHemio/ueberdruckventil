# ueberdruckventil

Unix socket pressure valve.

Reads from a (preexisting) Unix datagram socket, then tries to write any received message into a (newly created) Unix datagram socket. Messages are passed through a queue with a size cap; when this queue becomes full, `ueberdruckventil` complains on stderr and throws away any further messages until there is space in the queue again.
