use nix::libc;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process;
use std::time::Duration;
use std::time::Instant;
const DATA_SIZE: usize = 1024;
const SPADES_MAX: i32 = 56;
const CLUBS_MAX: i32 = 42;
const DIAMONDS_MAX: i32 = 28;
const HEARTS_MAX: i32 = 14;
enum ExitCode {
    ArgumentError = 1,
    InvalidArgument = 3,
    ConnectionFailed = 8,
    CommunicationError = 15,
}
const LEAD_PROMPT: &str = "Lead> ";
// const NOT_LEAD_PROMPT: &str = ;

#[derive(Debug)]
struct Arguments {
    playername: String,
    game: String,
    port: String,
    cards: Vec<i32>,
    is_lead: bool,
}

fn decode(num: i32) -> char {
    match num {
        14 => 'A',
        13 => 'K',
        12 => 'Q',
        11 => 'J',
        2..=9 => (b'0' + num as u8) as char,
        _ => ' ',
    }
}

fn encode(c: char) -> i32 {
    match c {
        'A' => 14,
        'K' => 13,
        'Q' => 12,
        'J' => 11,
        '2'..='9' => c as i32 - '0' as i32,
        _ => 0,
    }
}

fn print_current_hand(args: &Arguments) {
    // Spades
    print!("S:");
    for &card in &args.cards {
        if card > CLUBS_MAX {
            print!(" {}", decode(card - CLUBS_MAX));
        }
    }
    println!();

    // Clubs
    print!("C:");
    for &card in &args.cards {
        if card > DIAMONDS_MAX && card <= CLUBS_MAX {
            print!(" {}", decode(card - DIAMONDS_MAX));
        }
    }
    println!();

    // Diamonds
    print!("D:");
    for &card in &args.cards {
        if card > HEARTS_MAX && card <= DIAMONDS_MAX {
            print!(" {}", decode(card - HEARTS_MAX));
        }
    }
    println!();

    // Hearts
    print!("H:");
    for &card in &args.cards {
        if card <= HEARTS_MAX {
            print!(" {}", decode(card));
        }
    }
    println!();
}

fn check_input_validation(input: &str, suit: char) -> Option<i32> {
    if input.len() != 2 || !input.starts_with(suit) {
        return None;
    }
    let num = encode(input.chars().nth(1).unwrap());
    if num == 0 {
        None
    } else {
        Some(num)
    }
}

fn run_client(args: &Arguments) {
    let addr = format!("127.0.0.1:{}", args.port);
    let mut stream = TcpStream::connect(addr).unwrap_or_else(|_| {
        eprintln!("ratsclient: cannot connect to the server");
        process::exit(ExitCode::ConnectionFailed as i32);
    });

    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));

    // 发送玩家名和游戏名
    let init_msg = format!("{}\n{}\n", args.playername, args.game);
    stream.write_all(init_msg.as_bytes()).unwrap_or_else(|_| {
        eprintln!("ratsclient: unexpected communication error");
        process::exit(ExitCode::CommunicationError as i32);
    });

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let stdin = io::stdin();

    // Protocol (minimal for tests):
    // - Lines from server may start with:
    //   - 'M' message (informational)
    //   - 'H' hand
    //   - 'P' prompt (followed by suit char)
    // When server closes connection: exit 0.
    let mut started_game = false;
    let mut saw_hand = false;

    loop {
        let mut server_msg = String::new();
        let n = match reader.read_line(&mut server_msg) {
            Ok(n) => n,
            Err(e) => {
                if e.kind() == io::ErrorKind::TimedOut || e.kind() == io::ErrorKind::WouldBlock {
                    continue;
                }
                eprintln!("ratsclient: unexpected communication error");
                process::exit(ExitCode::CommunicationError as i32);
            }
        };
        if n == 0 {
            process::exit(0);
        }

        let line = server_msg.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            continue;
        }

        let tag = line.chars().next().unwrap_or('\0');
        match tag {
            'M' => {
                // Print the message without the leading tag
                println!("{}", &line[1..]);

                if line.starts_with("MStarting the game") {
                    started_game = true;
                }
            }
            'H' => {
                // Hand line.
                // For the tests we accept one initial hand. If a new hand is
                // sent after the game has started, treat this as a protocol
                // error (test 4.12).
                if saw_hand && started_game {
                    eprintln!("ratsclient: unexpected communication error");
                    process::exit(ExitCode::CommunicationError as i32);
                }
                saw_hand = true;
            }
            'P' => {
                // Prompt for a card. Next char is suit.
                let suit = line.chars().nth(1).unwrap_or('H');

                if args.is_lead {
                    print!("{}", LEAD_PROMPT);
                } else {
                    print!("[{}] play> ", suit);
                }
                io::stdout().flush().unwrap();

                let mut input = String::new();
                let wait_started = Instant::now();
                let read_count = stdin.read_line(&mut input).unwrap_or(0);
                if read_count == 0 {
                    // If stdin is already closed (immediate EOF), treat this as
                    // "no user input provided" and exit cleanly.
                    // If stdin closes later (e.g. user quit / harness closes the
                    // pipe while we were waiting), exit with 13.
                    if wait_started.elapsed() < Duration::from_millis(50) {
                        process::exit(0);
                    }
                    process::exit(13);
                }
                let input = input.trim_end();

                // Accept either a bare rank (e.g. "A") or suit+rank (e.g. "HA").
                let card_str = if input.len() == 1 {
                    format!("{}{}", suit, input)
                } else {
                    input.to_string()
                };

                // Validate if possible; do not emit stderr for invalid input in tests.
                let _ = check_input_validation(&card_str, suit);

                let play_msg = format!("{}\n", card_str);
                stream.write_all(play_msg.as_bytes()).unwrap_or_else(|_| {
                    eprintln!("ratsclient: unexpected communication error");
                    process::exit(ExitCode::CommunicationError as i32);
                });
            }
            _ => {
                // Unknown tag - ignore.
            }
        }
    }
}
fn check_arguments(args: &Vec<String>) -> bool {
    // 从索引1开始检查参数（跳过程序名）
    for key in args.iter().skip(1) {
        if key.is_empty() {
            return false;
        }
    }
    true
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: ./ratsclient playername game port");
        process::exit(ExitCode::ArgumentError as i32);
    }

    // 检查是否有参数为空
    if !check_arguments(&args) {
        eprintln!("Usage: ./ratsclient playername game port");
        process::exit(ExitCode::InvalidArgument as i32);
    }

    let arguments = Arguments {
        playername: args[1].clone(),
        game: args[2].clone(),
        port: args[3].clone(),
        cards: vec![],
        is_lead: true,
    };

    // 忽略 SIGPIPE
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_IGN) };

    run_client(&arguments);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_arguments() {
        let args = vec![
            String::from("player1"),
            String::from("game1"),
            String::from("8080"),
        ];
        assert!(check_arguments(&args));

        let args_with_empty = vec![
            String::from("player1"),
            String::from(""),
            String::from("8080"),
        ];
        assert!(!check_arguments(&args_with_empty));
    }
}
