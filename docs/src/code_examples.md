# 代码示例

本章节提供服务器核心功能的代码示例，帮助开发者理解关键实现细节。

## 服务器初始化

以下是服务器初始化的关键代码：

```rust
fn init_server(port: &str) -> Result<TcpListener, std::io::Error> {
    // 创建TCP监听器
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    println!("服务器启动在端口 {}", port);
    Ok(listener)
}
```

## 处理客户端连接

```rust
fn handle_accept(ctx: &Arc<Mutex<ServerContext>>, listener: &TcpListener) -> Result<(), std::io::Error> {
    // 接受客户端连接
    let (stream, _) = listener.accept()?;
    let socket_fd = stream.as_raw_fd();
    
    // 创建客户端信息
    let client_info = create_client_info(socket_fd);
    
    // 将客户端添加到上下文
    add_client_to_context(ctx, client_info.clone());
    
    // 启动处理线程
    start_worker_thread(move || {
        // 线程处理逻辑
    });
    
    Ok(())
}
```

## 游戏设置

```rust
fn setup_full_game(game: &mut GameInfo) {
    // 初始化游戏状态
    game.current_turn = 0;
    game.leading_player = 0;
    game.suit = ' ';
    game.count = 0;
    game.team_one_tricks = 0;
    game.team_two_tricks = 0;
    
    // 初始化玩家的牌
    deal_cards(game);
    
    // 通知所有玩家游戏开始
    broadcast_game_start(game);
}
```

## 玩家出牌处理

```rust
fn update_play_state(game: &mut GameInfo, player_idx: usize, card: i32) {
    // 记录玩家出牌
    game.play_cards[player_idx] = card;
    game.count += 1;
    
    // 如果是第一个出牌的玩家，设置花色
    if game.count == 1 {
        game.leading_player = player_idx;
        // 从牌中提取花色信息
        let suit_char = decode_suit(card as u8);
        game.suit = suit_char;
    }
    
    // 检查是否所有玩家都已出牌
    if game.count == game.player_count {
        // 处理这一轮的结果
        process_trick(game);
    }
}

fn decode_card(card: u8) -> u8 {
    // 解码卡片值
    card % 13
}
```

## 条件变量使用示例

```rust
fn notify_condition_variable(cond: &Arc<Condvar>, lock: &Arc<Mutex<bool>>) {
    // 获取锁并通知所有等待的线程
    let _lock_guard = lock.lock().unwrap();
    cond.notify_all();
}
```

## 错误处理

```rust
fn handle_client_error(client_info: &Arc<Mutex<ClientInfo>>, error: std::io::Error) {
    // 记录错误
    eprintln!("客户端错误: {}", error);
    
    // 清理客户端资源
    cleanup_client(client_info);
}
```

## 线程创建

```rust
fn create_thread_args(fd: i32, ctx: Arc<Mutex<ServerContext>>, client_info: Arc<Mutex<ClientInfo>>, message: String) -> ThreadArgs {
    ThreadArgs {
        fd,
        ctx,
        client_info,
        message,
    }
}
```