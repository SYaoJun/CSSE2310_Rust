# 实现细节

本章节深入介绍TCP服务器的关键实现细节，包括网络通信、并发控制、数据结构等方面的具体设计和实现。

## 网络通信实现

### TCP监听器设置

服务器使用标准库中的`TcpListener`来监听客户端连接：

```rust
use std::net::TcpListener;

fn setup_tcp_listener(port: &str) -> std::io::Result<TcpListener> {
    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(address)?;
    listener.set_nonblocking(false)?; // 阻塞模式
    Ok(listener)
}
```

### 客户端连接处理

每个客户端连接都通过独立的线程进行处理：

```rust
fn handle_client_connections(ctx: Arc<Mutex<ServerContext>>, listener: TcpListener) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let ctx_clone = ctx.clone();
                let client_info = create_client_info(stream.as_raw_fd());
                
                // 添加客户端到上下文中
                {
                    let mut ctx_guard = ctx_clone.lock().unwrap();
                    ctx_guard.clients.push(client_info.clone());
                }
                
                // 启动处理线程
                std::thread::spawn(move || {
                    if let Err(e) = process_client_commands(client_info, stream) {
                        eprintln!("处理客户端命令失败: {}", e);
                        cleanup_client(client_info, ctx_clone);
                    }
                });
            }
            Err(e) => {
                eprintln!("接受连接失败: {}", e);
            }
        }
    }
}
```

## 并发控制机制

### 互斥锁使用模式

服务器代码中使用了多种互斥锁模式来保护共享数据：

#### 基本锁模式

```rust
// 获取锁并修改数据
let mut guard = mutex.lock().unwrap();
guard.field = new_value;
```

#### 作用域锁模式

使用花括号创建作用域，确保锁在不需要时及时释放：

```rust
{
    let mut game_guard = game.lock().unwrap();
    // 修改游戏状态
    game_guard.state = State::PLAYING;
}
// 锁已释放，其他线程可以访问
```

### 条件变量使用

条件变量用于线程间的同步通知，特别是在等待特定事件时：

```rust
fn wait_for_condition(cond: &Arc<Condvar>, mutex: &Arc<Mutex<bool>>, predicate: impl Fn() -> bool) {
    let mut guard = mutex.lock().unwrap();
    while !predicate() {
        guard = cond.wait(guard).unwrap();
    }
}
```

## 数据结构设计

### 客户端信息管理

```rust
struct ClientInfo {
    fd: i32,
    is_ready: bool,
    username: String,
    cards: Vec<i32>,
    team: i32,
    player_idx: usize,
    game: Option<Arc<Mutex<GameInfo>>>,
    socket: TcpStream,
}

impl ClientInfo {
    fn new(fd: i32, socket: TcpStream) -> Self {
        ClientInfo {
            fd,
            is_ready: false,
            username: String::new(),
            cards: Vec::new(),
            team: 0,
            player_idx: 0,
            game: None,
            socket,
        }
    }
    
    fn send_message(&mut self, message: &str) -> std::io::Result<()> {
        self.socket.write_all(message.as_bytes())?;
        Ok(())
    }
}
```

### 服务器上下文

```rust
struct ServerContext {
    games: Vec<Arc<Mutex<GameInfo>>>,
    clients: Vec<Arc<Mutex<ClientInfo>>>,
    next_client_id: usize,
}

impl ServerContext {
    fn new() -> Self {
        ServerContext {
            games: Vec::new(),
            clients: Vec::new(),
            next_client_id: 0,
        }
    }
    
    fn create_new_game(&mut self) -> Arc<Mutex<GameInfo>> {
        let game = Arc::new(Mutex::new(GameInfo::new()));
        self.games.push(game.clone());
        game
    }
}
```

## 错误处理策略

### 网络错误处理

```rust
fn handle_network_error(err: std::io::Error) {
    match err.kind() {
        std::io::ErrorKind::ConnectionReset => {
            eprintln!("连接被重置");
        },
        std::io::ErrorKind::BrokenPipe => {
            eprintln!("管道已断开");
        },
        std::io::ErrorKind::TimedOut => {
            eprintln!("连接超时");
        },
        _ => {
            eprintln!("网络错误: {}", err);
        }
    }
}
```

### 资源清理

当客户端断开连接或发生错误时，需要清理相关资源：

```rust
fn cleanup_client(client_info: Arc<Mutex<ClientInfo>>, ctx: Arc<Mutex<ServerContext>>) {
    // 从游戏中移除客户端
    if let Some(game_arc) = { 
        let mut client_guard = client_info.lock().unwrap();
        client_guard.game.take()
    } {
        let mut game_guard = game_arc.lock().unwrap();
        if let Some(player_idx) = game_guard.players.iter().position(|p| {
            p.as_ref().map_or(false, |c| {
                let client = c.lock().unwrap();
                client.fd == client_info.lock().unwrap().fd
            })
        }) {
            game_guard.players[player_idx] = None;
        }
    }
    
    // 从上下文中移除客户端
    {
        let mut ctx_guard = ctx.lock().unwrap();
        ctx_guard.clients.retain(|c| {
            let client = c.lock().unwrap();
            client.fd != client_info.lock().unwrap().fd
        });
    }
}
```

## 代码优化建议

1. **使用tokio异步运行时**：考虑使用tokio等异步运行时替代线程池，以提高并发性能
2. **错误处理优化**：使用Result和?运算符简化错误处理
3. **减少锁的粒度**：进一步减小锁的作用域，避免长时间持有锁
4. **使用无锁数据结构**：对于频繁读取的数据，考虑使用无锁数据结构
5. **添加日志系统**：集成结构化日志系统，便于问题排查和性能监控