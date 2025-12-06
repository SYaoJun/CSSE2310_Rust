use std::io::{Write};
use std::net::TcpListener;
use std::sync::{Arc, Condvar, Mutex, atomic::{AtomicUsize, Ordering}};
use std::thread;

use std::io;
// 导入第三方库
use signal_hook::iterator;
use signal_hook::consts;

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
     let context_info = Arc::new(Mutex::new(ContextInfo{
        current_connected: 0,
        total_connected: 0,
        running_games: 0,
        games_completed: 0,
        games_terminated: 0,
        total_tricks: 0,
    }));
    
    println!("进程ID: {}", std::process::id());
    println!("发送SIGHUP信号: kill -HUP {}", std::process::id());
    
    // 在单独的线程中处理信号
    let context_info_clone_sig = context_info.clone();
    thread::spawn(move || {
        // 使用signal_hook库注册SIGHUP信号处理
        let mut signal_receiver = iterator::Signals::new(&[consts::SIGHUP])
            .expect("Failed to register signal handler");
        
        // 无限循环等待信号
        for signal in signal_receiver.forever() {
            if signal == consts::SIGHUP {
                let context_info_guard = context_info_clone_sig.lock().unwrap();
                println!("Players connected: {}", context_info_guard.current_connected);
                println!("Total connected players: {}", context_info_guard.total_connected);
                println!("Running games: {}", context_info_guard.running_games);
                println!("Games completed: {}", context_info_guard.games_completed);
                println!("Games terminated: {}", context_info_guard.games_terminated);
                println!("Total tricks: {}", context_info_guard.total_tricks);
            }
        }
    });
    
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
   
    loop{
        let stream = listener.accept().unwrap().0;
        let client_info = Arc::new(Mutex::new(ClientInfo::new(stream)));
        // 这里是否需要clone 这个 client_info 到新线程中
        let client_info_clone = client_info.clone();
        let context_info_clone = context_info.clone();
        std::thread::spawn(move || {
            if let Err(e) = handle_new_connection( client_info_clone, context_info_clone) {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
    
}
