# 连接处理

连接处理模块负责接受、管理和关闭客户端连接，是服务器与客户端交互的基础。

## 主要功能

### 监听连接

服务器通过TCP套接字监听指定端口，接受来自客户端的连接请求。

```rust
// 创建TCP监听器的伪代码
let listener = TcpListener::bind("127.0.0.1:8888")?;
for stream in listener.incoming() {
    // 处理每个新连接
}
```

### 连接建立

当客户端发起连接时，服务器接受连接并创建对应的ClientInfo结构，记录客户端的状态信息。

### 数据传输

使用Unix socket或TCP连接与客户端进行数据交换，处理命令和响应。

### 连接关闭

当客户端断开连接或发生错误时，服务器会清理相关资源，从游戏房间中移除玩家，并通知其他玩家。

## 关键数据结构

### ClientInfo

存储客户端的连接状态、玩家信息和游戏相关数据。

```rust
struct ClientInfo {
    next: Option<Arc<Mutex<ClientInfo>>>,
    conn: Option<Arc<Mutex<UnixStream>>>,
    player_name: String,
    state: State,
    socket: i32,
    game_name: Option<String>,
    player_idx: usize,
    cards: Vec<i32>,
    team: i32,
    ready: bool,
}
```

## 连接管理流程

1. **初始化阶段**: 创建监听器并设置服务器状态
2. **接受阶段**: 监听并接受新的客户端连接
3. **处理阶段**: 为每个连接创建工作线程处理请求
4. **清理阶段**: 客户端断开连接后释放资源

## 异常处理

- 网络连接失败的错误捕获和日志记录
- 客户端异常断开的优雅处理
- 资源泄露的预防措施