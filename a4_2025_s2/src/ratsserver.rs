use std::io::{Write, Read};
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
    max_connections: i32,
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

fn handle_new_connection( client_info: ClientInfo, context_info: Arc<Mutex<ContextInfo>>) -> std::io::Result<()> {
    let mut stream = client_info.tcp_stream.try_clone().unwrap();
    let mut buffer = [0; 1024];
    // 打印当前线程 id
    println!("当前线程ID: {:?}", thread::current().id());
    loop{
        match stream.read(&mut buffer) {
            Ok(0) => {
                // 读到0字节，表示连接已断开
                println!("客户端断开连接");
                break;
            }
            Ok(n) => {
                // 正常读取到数据
                println!("收到 {} 字节数据", n);
                // 处理数据...
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // 非阻塞模式下的无数据可读
                continue;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // 读取超时
                println!("读取超时");
                continue;
            }
            Err(e) => {
                // 其他错误，通常表示连接断开
                println!("读取错误: {}, 连接可能已断开", e);
                break;
            }
        } 
    }
    let mut context_info_guard = context_info.lock().unwrap();
    context_info_guard.current_connected -= 1;  
    context_info_guard.games_completed += 1;
    Ok(())
}

/// 启动服务器的函数
/// 
/// # 参数
/// * `port` - 服务器监听的端口
/// * `maxconns` - 最大连接数
/// 
/// # 返回值
/// * `Ok(())` - 服务器正常启动
/// * `Err(e)` - 服务器启动失败
fn start_server(port: &str, maxconns: usize) -> io::Result<()> {
    let context_info = Arc::new(Mutex::new(ContextInfo{
        current_connected: 0,
        total_connected: 0,
        running_games: 0,
        games_completed: 0,
        games_terminated: 0,
        total_tricks: 0,
        max_connections: maxconns as i32,
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
    
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
   
    loop{
        let stream = listener.accept()?.0;
        let client_info = ClientInfo::new(stream);
        let context_info_clone = context_info.clone();
        {
            let mut lock = context_info_clone.lock().unwrap();
            if lock.current_connected >= lock.max_connections as i32 {
                drop(lock);
                continue;
            }
             lock.current_connected += 1;
            lock.total_connected += 1;
        }
        std::thread::spawn(move || {
            if let Err(e) = handle_new_connection( client_info, context_info_clone) {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}

fn main()->io::Result<()>  {
    // 接受命令行参数
    let args: Vec<String> = std::env::args().collect();
    let maxconns = args.get(1).map(|s| s.parse().unwrap_or(10)).unwrap_or(10);
    
    // 处理端口参数，避免临时值被过早释放
    let default_port = "8080".to_string();
    let port = args.get(2).unwrap_or(&default_port);
    
    start_server(port, maxconns)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;
    use std::time::Duration;
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;
    use std::thread;
    use std::thread::JoinHandle;

    // 运行测试的辅助函数，带有超时
    fn run_test_with_timeout<F>(test_func: F, timeout: Duration) -> Result<(), String>
    where
        F: FnOnce() -> (),
        F: Send + 'static,
    {
        use std::sync::mpsc;

        // 创建通道用于通知测试完成
        let (tx, rx) = mpsc::channel();

        // 在新线程中运行测试函数
        thread::spawn(move || {
            // 运行测试函数
            test_func();
            // 测试完成后发送信号
            let _ = tx.send(());
        });

        // 等待测试完成或超时
        match rx.recv_timeout(timeout) {
            Ok(_) => Ok(()),
            Err(mpsc::RecvTimeoutError::Timeout) => Err("Test timed out".to_string()),
            Err(_) => Err("Test failed with unknown error".to_string()),
        }
    }

    // 测试服务器能够成功接受客户端连接
    #[test]
    fn test_server_accepts_connections() {
        let test_result = run_test_with_timeout(
            || {
                // 使用一个不太可能被占用的端口
                let test_port = "8888";
                let maxconns = 5;

                // 在新线程中启动服务器
                let handle = thread::spawn(move || {
                    let _ = start_server(test_port, maxconns);
                });

                // 给服务器一点时间启动
                thread::sleep(Duration::from_millis(100));

                // 尝试连接服务器
                let result = TcpStream::connect(format!("127.0.0.1:{}", test_port));
                
                // 验证连接成功
                assert!(result.is_ok(), "Failed to connect to server");
                
                // 关闭连接
                if let Ok(mut stream) = result {
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                }

                // 注意：这里我们没有优雅地关闭服务器
                let _ = handle.join();
            },
            Duration::from_secs(5), // 5秒超时
        );

        // 检查测试是否超时
        assert!(test_result.is_ok(), "{}", test_result.unwrap_err());
    }

    // 测试端口被占用时的错误处理
    #[test]
    fn test_server_port_in_use() {
        let test_result = run_test_with_timeout(
            || {
                // 使用同一个端口启动两个服务器
                let test_port = "8889";
                let maxconns = 5;

                // 第一个服务器应该能正常启动
                let handle1 = thread::spawn(move || {
                    let _ = start_server(test_port, maxconns);
                });

                // 给第一个服务器一点时间启动
                thread::sleep(Duration::from_millis(100));

                // 第二个服务器应该返回错误
                let result = std::net::TcpListener::bind(format!("127.0.0.1:{}", test_port));
                
                // 验证端口被占用
                assert!(result.is_err(), "Expected port to be in use, but it wasn't");
                
                // 检查错误类型是否为地址已在使用
                if let Err(e) = result {
                    assert_eq!(e.kind(), std::io::ErrorKind::AddrInUse, "Expected AddrInUse error");
                }

                // 注意：这里我们没有优雅地关闭第一个服务器
                let _ = handle1.join();
            },
            Duration::from_secs(5), // 5秒超时
        );

        // 检查测试是否超时
        assert!(test_result.is_ok(), "{}", test_result.unwrap_err());
    }

    // 测试服务器能够正确处理 SIGHUP 信号
    #[test]
    fn test_server_handles_sighup() {
        let test_result = run_test_with_timeout(
            || {
                // 使用一个不太可能被占用的端口
                let test_port = "8890";
                let maxconns = 5;

                // 在新线程中启动服务器
                let handle = thread::spawn(move || {
                    let _ = start_server(test_port, maxconns);
                });

                // 给服务器一点时间启动
                thread::sleep(Duration::from_millis(100));

                // 获取当前进程ID（服务器在同一进程中运行）
                let pid = Pid::from_raw(std::process::id() as i32);

                // 向服务器发送 SIGHUP 信号
                let result = signal::kill(pid, Signal::SIGHUP);
                
                // 验证信号发送成功
                assert!(result.is_ok(), "Failed to send SIGHUP signal");

                // 给服务器一点时间处理信号
                thread::sleep(Duration::from_millis(100));

                // 验证服务器仍然在运行（可以继续接受连接）
                let connection_result = TcpStream::connect(format!("127.0.0.1:{}", test_port));
                assert!(connection_result.is_ok(), "Server crashed after receiving SIGHUP signal");
                
                // 关闭连接
                if let Ok(mut stream) = connection_result {
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                }

                // 注意：这里我们没有优雅地关闭服务器
                let _ = handle.join();
            },
            Duration::from_secs(5), // 5秒超时
        );

        // 检查测试是否超时
        assert!(test_result.is_ok(), "{}", test_result.unwrap_err());
    }
}
