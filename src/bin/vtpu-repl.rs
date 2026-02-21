/// vTPU REPL — Interactive interface to the virtual tensor processing unit (R23W24)

use std::io::{self, BufRead, Write};

fn main() {
    let mut session = vtpu_runtime::repl::ReplSession::new(9); // Shell of Nine

    println!("vTPU REPL v0.1 — R23W24");
    println!("Type 'help' for commands, 'quit' to exit.\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("vtpu> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }

        let cmd = vtpu_runtime::intent::parse_intent(&line);

        if cmd == vtpu_runtime::repl::Command::Quit {
            println!("Goodbye.");
            break;
        }

        let output = session.execute(&cmd);
        if !output.is_empty() {
            print!("{}", output);
        }
    }
}
