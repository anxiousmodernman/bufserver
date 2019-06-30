use std::thread;
use std::net::{TcpListener, TcpStream};
use std::os::unix::net::{UnixStream, UnixListener};
use std::time::Duration;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::io::{BufReader, BufRead};
use std::fs;
use log_buffer::LogBuffer;
use std::fmt::Write;
use std::time::Instant;


/* Debug with this program by connecting to the unix socket and reading until EOF.
 * It is okay to read multiple times. You can use socat to read:
 *     socat -u UNIX-CONNECT:/tmp/debug-logstash - | less
 */

// I guess this works for \r\n too.
const NEWLINE: u8 = 0xA;

// 5 MB
const CAPACITY: usize = 5242880;

fn main() {

    let (tx, rx) = channel::<Vec<u8>>();
    let mut buf = LogBuffer::new(vec![0; CAPACITY]); // why not Vec::new ?
    write!(buf, "\n"); // writing a newline first prevents the first line from being dropped
    let mtx = Mutex::new(buf);
    let arc_buf = Arc::new(mtx);

    let arc_buf1 = arc_buf.clone();
    let arc_buf2 = arc_buf.clone();

    thread::spawn(move ||fill_buffer(rx, arc_buf1));

    thread::spawn(move || {
        let path = "/tmp/debug-logstash";
        if path_exists(path) {
            fs::remove_file(path);
        }
        let ul = UnixListener::bind(path).unwrap();

        // We must scope this trait import here :-/
        use std::io::Write;

        for stream in ul.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut lock = arc_buf2.lock().unwrap();
                    let lines: Vec<&str> = lock.extract_lines().collect();
                    for l in lines {
                        stream.write(l.as_bytes());
                        stream.write(b"\n");
                    }
                },
                Err(e) => panic!("got error accepting unix socket connection: {}", e)
            }
        }
    });

    let listener = TcpListener::bind("127.0.0.1:9595").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let txx = tx.clone();
                thread::spawn(move || handle_tcp_client(stream, txx));
            },
            Err(e) => panic!("got an error accepting connection: {}", e),
        }
    }

}

fn fill_buffer(rx: Receiver<Vec<u8>>, buf: Arc<Mutex<LogBuffer<Vec<u8>>>>) {
    loop {
        let v = rx.recv().unwrap();
        {
            let mut lock = buf.lock().unwrap();
            let msg = format!("{}", String::from_utf8(v).unwrap_or(format!("invalid utf-8")));
            lock.write_str(&msg);
        }
    }
}

fn handle_tcp_client(stream: TcpStream, tx: Sender<Vec<u8>>) {

    println!("connection from tcp {:?}", stream.peer_addr().unwrap());
    let mut br = BufReader::new(stream);
    loop {
        let mut line = Vec::new();
        match br.read_until(NEWLINE, &mut line) {
            Ok(n) if n == 0 => {
                // EOF, conn closed (probably)
                println!("connection closed");
                return;
            }
            Ok(n) => {
//                println!("sending {}", n);
                tx.send(line);
            }
            Err(e) => {
                println!("error: {:?}", e);
            }
        };
    }
}


pub fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}