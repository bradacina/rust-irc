extern crate mio;

use mio::net::TcpStream;
use mio::{Events, Poll, PollOpt, Ready, Token};
use std::io::{Error, ErrorKind, Read, Write};
use std::net::SocketAddr;
use std::string::String;
use std::time::Duration;

struct Stuff {
    poll: Poll,
    events: Events,
    sock: TcpStream,
}

enum Command<'a> {
    None,
    Ping(&'a str)
}

impl Stuff {
    fn my_poll(&mut self) {
        self.poll
            .poll(&mut self.events, Some(Duration::from_millis(100)))
            .unwrap();
    }

    fn poll_until_writable(&mut self) {
        loop {
            self.my_poll();

            for event in &self.events {
                if event.readiness().is_writable() {
                    return;
                }
            }
        }
    }

    fn my_read(&mut self, buf: &mut String) -> Result<(), Error> {
        buf.clear();
        for event in &self.events {
            if event.readiness().is_readable() {
                let _ = self.sock.read_to_string(buf);
                return Ok(());
            }
        }

        return Err(Error::new(ErrorKind::Other, "Socket is not readable yet"));
    }

    fn my_write(&mut self, buf: &mut String) -> Result<(), Error> {
        for event in &self.events {
            if event.readiness().is_writable() {
                println!("socket is writable. will write {}", buf);
                self.sock.write(buf.as_bytes())?;
                buf.clear();
                return Ok(());
            }
        }

        return Err(Error::new(ErrorKind::Other, "Socket is not writable yet"));
    }
}

fn parse_command(cmd: &str) -> Command {
    const PING_COMMAND: &str = "PING";

    let mut split = cmd.split_whitespace();

    match split.next() {
        Some(PING_COMMAND) => {
            Command::Ping(split.next().unwrap())
        },
        _ => Command::None
    }
}

fn handle_command(cmd: Command, stuff: &mut Stuff) {
    let pong_command = "PONG";
    let space_command = " ";
    let eol_command = "\r\n";

    let mut write_buf = String::new();
    
    match cmd {
        Command::Ping(token) => {
            write_buf.push_str(pong_command);
            write_buf.push_str(space_command);
            write_buf.push_str(token);
            write_buf.push_str(eol_command);
            let _ = stuff.my_write(&mut write_buf);
        },

        Command::None => ()
    }
}

fn main() {
    // setup the user command
    let user_command = "USER something_special 0 * :Something Special\r\n";
    let nick_command = "NICK something_special123\r\n";

    // setup the address of the server
    // todo: resolve dns?
    let addr: SocketAddr = "54.219.165.167:6667".parse().unwrap();

    // Setup the client socket
    let sock = TcpStream::connect(&addr).unwrap();

    // Create a poll instance
    let poll = Poll::new().unwrap();

    poll.register(
        &sock,
        Token(0),
        Ready::readable() | Ready::writable(),
        PollOpt::edge(),
    ).unwrap();

    let events = Events::with_capacity(1024);
    let mut read_buf = String::new();

    let mut stuff = Stuff {
        poll: poll,
        events: events,
        sock: sock,
    };

    let mut write_buf = String::new();

    stuff.poll_until_writable();
    write_buf.push_str(user_command);

    let _ = stuff.my_write(&mut write_buf);

    write_buf.push_str(nick_command);
    let _ = stuff.my_write(&mut write_buf);

    loop {
        stuff.my_poll();

        if stuff.my_read(&mut read_buf).is_ok() {
            println!("{}", read_buf);

            let command = parse_command(&read_buf);

            handle_command(command, &mut stuff);
        }
    }
}
