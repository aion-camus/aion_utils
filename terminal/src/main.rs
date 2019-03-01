use std::io::{self, Write};
use std::thread;
use std::sync::mpsc::{channel, Sender};

mod parser;

use parser::AppOptions;

fn start(tx: Sender<i32>, info: String) {
    let apps = AppOptions::new("avm");
    match thread::Builder::new().name("cmd".to_string()).spawn(move || {
        loop {
            print!("{}", info);
            io::stdout().flush().unwrap();
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n == 0 {
                    tx.send(0).unwrap();
                    break;
                }
                input.pop();
                match input.as_str() {
                    _ => {
                        println!("{} bytes read", n);
                        println!("{}", input);
                    },
                }
                parser::parse(&input, &apps);
            }
            Err(error) => println!("error: {}", error),
            }
        }
    }) {
        Ok(_) => println!("new thread cmd"),
        Err(x) => println!("Create thread {} failed: {:?}", "cmd", x),
    }
}

fn main() {
    // commandline info
    let cmd_line = "kernel-v0.1.0 >> ".to_string();

    let (tx, rx) = channel::<i32>();
    start(tx, cmd_line);

    // wait for thread signals
    let r = rx.recv().unwrap();

    println!("main exits: {:?}", r);
}