#!/usr/bin/env python3
import argparse
import random
import socket
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-s", "--stop-listening",
        dest="stop_listening", action="store_true",
    )
    parser.add_argument(
        "-r", "--restart-listening",
        dest="restart_listening", action="store_true",
    )
    parser.add_argument(
        dest="incoming_socket",
    )
    args = parser.parse_args()

    sock = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM, 0)
    sock.bind(args.incoming_socket)
    while True:
        msg = sock.recv(10*1024*1024)
        print("received a message")
        if args.stop_listening:
            # become obstinate?
            n = random.randint(0, 1023)
            if n == 0:
                print("that is quite enough")
                while True:
                    time.sleep(1)
                    if args.restart_listening:
                        n = random.randint(0, 3)
                        if n == 0:
                            print("alright, I will listen again")
                            break
                        else:
                            print("still not listening")


if __name__ == "__main__":
    main()
