use std::os::unix::net::UnixDatagram;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use clap::Parser;


#[derive(Parser)]
struct Opts {
    #[arg(short = 'q', long, default_value = "4096")]
    pub max_queue_length: usize,

    #[arg(short = 'm', long, default_value = "10485760")] // 10 MiB
    pub max_message_length: usize,

    pub pull_socket_path: PathBuf,
    pub push_socket_path: PathBuf,
}


fn pull_proc(pull_socket_path: PathBuf, max_message_length: usize, sender: mpsc::SyncSender<Vec<u8>>) {
    // prepare a buffer
    let mut buf = vec![0u8; max_message_length];

    loop {
        // bind the pull socket
        let pull_socket = UnixDatagram::bind(&pull_socket_path)
            .expect("failed to bind to the pull socket");

        // as long as the socket is okay
        loop {
            // receive a message (this may block, but that's fine)
            let read_count = match pull_socket.recv(&mut buf) {
                Err(e) => {
                    eprintln!("error reading from pull socket: {}", e);

                    // break out of the inner loop into the outer one, recreating the socket
                    break;
                },
                Ok(0) => {
                    eprintln!("zero read from pull socket");
                    break;
                },
                Ok(rc) => rc,
            };

            // copy it into a buffer we can send over
            let mut single_msg = Vec::with_capacity(read_count);
            single_msg.extend_from_slice(&buf[0..read_count]);

            // try to send it
            if let Err(e) = sender.try_send(single_msg) {
                match e {
                    mpsc::TrySendError::Full(_) => {
                        eprintln!("queue is full! dropping a message");
                        continue;
                    },
                    mpsc::TrySendError::Disconnected(_) => {
                        eprintln!("push thread is gone; I give up");
                        return;
                    },
                }
            }
        }
    }
}

fn push_proc(push_socket_path: PathBuf, receiver: mpsc::Receiver<Vec<u8>>) {
    // we won't try to compensate for timeouts, blocking behavior etc. in this function;
    // we let pull_proc fail on try_send

    let mut push_socket = None;
    loop {
        // receive a message
        let msg = match receiver.recv() {
            Ok(m) => m,
            Err(_) => {
                eprintln!("pull thread is gone; I give up");
                return;
            },
        };

        // do we have a working socket?
        if push_socket.is_none() {
            let new_socket = match UnixDatagram::unbound() {
                Ok(ns) => ns,
                Err(e) => {
                    eprintln!("failed to create an unbound Unix datagram socket: {} -- dropping a message", e);
                    continue;
                },
            };
            if let Err(e) = new_socket.connect(&push_socket_path) {
                eprintln!("failed to connect Unix datagram socket to push socket: {} -- dropping a message", e);
                continue;
            }
            push_socket = Some(new_socket);
        }

        match push_socket.as_ref().unwrap().send(&msg) {
            Ok(sent) => {
                if sent != msg.len() {
                    eprintln!("incomplete message sent ({} provided, {} transmitted)!", msg.len(), sent);

                    // recreate our socket the next time around
                    push_socket = None;
                }
            },
            Err(e) => {
                eprintln!("failed to send {} bytes of data to push socket: {} -- potentially dropping a message", msg.len(), e);
                push_socket = None;
            },
        }
    }
}


fn main() {
    let opts = Opts::parse();

    // create the channel between the two threads
    let (sender, receiver) = mpsc::sync_channel(opts.max_queue_length);

    // run pulling on its own thread
    thread::spawn(move || pull_proc(opts.pull_socket_path, opts.max_message_length, sender));

    // run pushing on the main thread
    push_proc(opts.push_socket_path, receiver);
}
