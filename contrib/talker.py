#!/usr/bin/env python3
import argparse
import socket
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        dest="outgoing_socket",
    )
    parser.add_argument(
        "-d", "--delay",
        dest="delay", type=float, default=0,
    )
    args = parser.parse_args()

    sock = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM, 0)
    sock.connect(args.outgoing_socket)
    base_bytes = bytes(range(256))
    full_buf = 4 * base_bytes
    while True:
        sock.send(full_buf)
        if args.delay:
            time.sleep(args.delay)


if __name__ == "__main__":
    main()
