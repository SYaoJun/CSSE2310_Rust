use std::io::{self, Write, BufRead, BufReader};
use std::net::{TcpStream, ToSocketAddrs};
use std::process;
use std::str;
use nix::libc;
const DATA_SIZE: usize = 1024;
const SPADES_MAX: i32 = 56;
const CLUBS_MAX: i32 = 42;
const DIAMONDS_MAX: i32 = 28;
const HEARTS_MAX: i32 = 14;

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
    let addr = format!("localhost:{}", args.port);
    let mut stream = TcpStream::connect(addr).unwrap_or_else(|_| {
        eprintln!("ratsclient: cannot connect to the server");
        process::exit(8);
    });

    // 发送玩家名和游戏名
    let init_msg = format!("{}\n{}\n", args.playername, args.game);
    stream.write_all(init_msg.as_bytes()).unwrap_or_else(|_| {
        eprintln!("ratsclient: unexpected communication error");
        process::exit(15);
    });

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut suit = 'H';
    let stdin = io::stdin();

    loop {
        let mut server_msg = String::new();
        if reader.read_line(&mut server_msg).unwrap_or(0) == 0 {
            eprintln!("ratsclient: unexpected communication error");
            process::exit(15);
        }
        println!("Info: {}", server_msg.trim_end());

        if args.is_lead {
            print!("{}", LEAD_PROMPT);
        } else {
            print!("[{}] play> ", suit);
        }
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if stdin.read_line(&mut input).unwrap_or(0) == 0 {
            eprintln!("ratsclient: user quit");
            process::exit(13);
        }
        let input = input.trim_end();

        if let Some(card_num) = check_input_validation(input, suit) {
            println!("Valid card: {}{}", suit, decode(card_num));
        } else {
            println!("Invalid input!");
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: ./ratsclient playername game port");
        process::exit(1);
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

