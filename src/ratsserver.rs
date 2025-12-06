use std::collections::{HashMap, VecDeque};
use std::io::{Write, Read};
use std::net::TcpListener;
use std::os::unix::signal::{self, Signal};
use std::os::fd::{AsRawFd};
use std::os::unix::net::UnixStream;
use std::str::FromStr;
use std::sync::{Arc, Condvar, Mutex, RwLock, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::thread;
use std::time::Duration;

use std::io;
// 导入第三方库
use signal_hook::flag::register;
use libc::{c_void};

// 简单的信号量实现
struct ConnectionSemaphore {
    count: AtomicUsize,
    max_count: usize,
    condvar: Condvar,
    mutex: Mutex<()>,
}

impl ConnectionSemaphore {
    fn new(max_count: usize) -> Self {
        ConnectionSemaphore {
            count: AtomicUsize::new(max_count),
            max_count,
            condvar: Condvar::new(),
            mutex: Mutex::new(()),
        }
    }
    
    fn try_acquire(&self) -> bool {
        let current = self.count.load(Ordering::SeqCst);
        if current > 0 {
            self.count.compare_exchange(current, current - 1, Ordering::SeqCst, Ordering::Relaxed).is_ok()
        } else {
            false
        }
    }
    
    fn release(&self) {
        self.count.fetch_add(1, Ordering::SeqCst);
        self.condvar.notify_one();
    }
}

// 常量定义
const DEFAULT_PORT: &str = "7777";
const MAX_PORT_NUMBER: u16 = 65535;
const BUFFER_SIZE: usize = 1024;
const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;
const DEFAULT_MESSAGE: &str = "Welcome to Rats!";
const MAX_CONNECTIONS: usize = 100;

// 常量定义
const LOCALHOST: &str = "localhost";
const MIN_ARGS_SIZE: usize = 3;
const MAX_ARGS_SIZE: usize = 4;
const MAX_INT_SIZE: usize = 5;
const MAX_CONNECT_NUM: usize = 10000;
const MAX_ROUND: usize = 13;
const MAX_TRICKS: usize = 13;
const CARD_SIZE: usize = 256;
const DATA_SIZE: usize = 1024;
const TEMP_SIZE: usize = 64;
const MAX_PLAYER_NUM: usize = 4;
const MAX_CARD_NUM: usize = 32;
const MAX_DECK_SIZE: usize = 104;
const SINGLE_CARD_GROUP: usize = 8;
const CARD_ACE: i32 = 14;
const CARD_KING: i32 = 13;
const CARD_QUEEN: i32 = 12;
const CARD_JACK: i32 = 11;
const CARD_TEN: i32 = 10;
const CARD_MIN_NUM: i32 = 2;
const CARD_MAX_NUM: i32 = 9;
const PLAYER_ZERO_MOD: usize = 0;
const PLAYER_ONE_MOD: usize = 2;
const PLAYER_TWO_MOD: usize = 4;
const PLAYER_THREE_MOD: usize = 6;
const TEAM_ONE_FIRST_PLAYER: usize = 0;
const TEAM_ONE_SECOND_PLAYER: usize = 2;
const TEAM_TWO_FIRST_PLAYER: usize = 1;
const TEAM_TWO_SECOND_PLAYER: usize = 3;
const PLAYER_THREE_INDEX: usize = 3;
const NAME_LENGTH: usize = 10;
const HAND_CARD_STEP: usize = 2;
const MIN_RECV_LEN: usize = 2;
const INITIAL_TURN: usize = 0;
const INITIAL_LEADING: usize = 0;
const INITIAL_PLAYER_IDX: usize = 0;
const MSG_NOSIGNAL: i32 = 0x00000008; // 模拟C的MSG_NOSIGNAL标志

// 退出状态码
const EXIT_SYSTEM_STATUS: i32 = 20;
const EXIT_PORT_STATUS: i32 = 17;
const EXIT_USAGE_STATUS: i32 = 8;

// 状态枚举
#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    IDLE = 0,
    WAITING = 1,
    READY = 2,
    PLAYING = 3,
    COMPLETED = 4,
}

// 命令行参数结构体
struct Arguments {
    maxconns: usize,
    port: Option<String>,
    message: String,
}
struct ContextInfo{
    current_connected: i32,
    total_connected: i32,
    running_games: i32,
    games_completed: i32,
    games_terminated: i32,
    total_tricks: i32,
}
// 客户端信息结构体
struct ClientInfo {
    next: Option<Arc<Mutex<ClientInfo>>>,
    tcp_stream: std::net::TcpStream,
    name: String,
    game_name: String,
    state: State,
    idx: usize,
    hand: String,
}
impl ClientInfo {
    fn new(stream: std::net::TcpStream) -> Self {
        ClientInfo {
            next: None,
            tcp_stream: stream,
            name: String::new(),    
            game_name: String::new(),
            state: State::IDLE,
            idx: 0,
            hand: String::new(),
        }
    }
}

fn handle_new_connection( client_info: Arc<Mutex<ClientInfo>>, context_info: Arc<Mutex<ContextInfo>>) -> std::io::Result<()> {
    let client_info_guard = client_info.lock().unwrap();
    let mut stream = client_info_guard.tcp_stream.try_clone().unwrap();
    stream.write(b"hello world from server").unwrap();
    let mut context_info_guard = context_info.lock().unwrap();
    context_info_guard.current_connected += 1;
    context_info_guard.total_connected += 1;
    Ok(())
}

fn main()->io::Result<()>  {
     let mut context_info = Arc::new(Mutex::new(ContextInfo{
        current_connected: 0,
        total_connected: 0,
        running_games: 0,
        games_completed: 0,
        games_terminated: 0,
        total_tricks: 0,
    }));
    let mut sig = signal::Signal::new(signal::SignalKind::hangup())?;
    
    println!("进程ID: {}", std::process::id());
    println!("发送SIGHUP信号: kill -HUP {}", std::process::id());
    
    // 在单独的线程中处理信号
    let context_info_clone_sig = context_info.clone();
    thread::spawn(move || {
        loop {
            // 等待信号
            sig.recv().expect("等待信号失败");
            /*
            Players connected: 4
            Total connected players: 4
            Running games: 1
            Games completed: 0
            Games terminated: 0
            Total tricks: 0
            */
            let mut context_info_guard = context_info_clone_sig.lock().unwrap();
            println!("Players connected: {}", context_info_guard.current_connected);
            println!("Total connected players: {}", context_info_guard.total_connected);
            println!("Running games: {}", context_info_guard.running_games);
            println!("Games completed: {}", context_info_guard.games_completed);
            println!("Games terminated: {}", context_info_guard.games_terminated);
            println!("Total tricks: {}", context_info_guard.total_tricks);
            
        }
    });
    
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
   
    loop{
        let stream = listener.accept().unwrap().0;
        let client_info = Arc::new(Mutex::new(ClientInfo::new(stream)));
        let client_info_clone = client_info.clone();
        let context_info_clone = context_info.clone();
        std::thread::spawn(move || {
            if let Err(e) = handle_new_connection( client_info_clone, context_info_clone) {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
    
}
