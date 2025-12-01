# 游戏状态管理

游戏状态管理模块负责维护游戏房间、玩家状态、游戏进行过程中的各种数据，并确保状态同步和一致性。

## 核心数据结构

### GameInfo

游戏信息结构体，存储游戏房间的所有状态信息：

```rust
struct GameInfo {
    next: Option<Arc<Mutex<GameInfo>>>,
    state: State,
    game_name: String,
    players: Vec<Option<Arc<Mutex<ClientInfo>>>>,
    player_count: usize,
    current_turn: usize,
    leading_player: usize,
    suit: char,
    play_cards: [i32; MAX_PLAYER_NUM],
    count: usize,
    team_one_tricks: usize,
    team_two_tricks: usize,
    count_ready: usize,
    cond: Arc<Condvar>,
    lock: Arc<Mutex<bool>>,
}
```

### State枚举

定义游戏的不同状态：

```rust
enum State {
    WAITING,
    READY,
    PLAYING,
    FINISHED,
}
```

## 游戏状态管理功能

### 游戏创建

创建新的游戏房间，初始化游戏状态和数据结构。

### 玩家加入

将玩家添加到游戏房间，并更新玩家信息和游戏状态。

### 游戏准备

等待所有玩家准备就绪，使用条件变量同步。

```rust
fn wait_all_ready(game: &mut GameInfo) -> bool {
    {
        let mut lock_guard = game.lock.lock().unwrap();
        while game.count_ready < game.player_count {
            lock_guard = game.cond.wait(lock_guard).unwrap();
        }
    }
    
    game.state = State::PLAYING;
    setup_full_game(game);
    true
}
```

### 游戏进行

管理回合顺序、卡片出牌、得分计算等游戏核心逻辑。

### 游戏结束

计算最终得分，确定获胜方，并通知所有玩家。

## 同步机制

- **互斥锁**: 保护共享数据的并发访问
- **条件变量**: 用于线程间的同步通知
- **原子操作**: 用于某些简单的状态更新

## 状态一致性保障

- 所有状态修改都在锁的保护下进行
- 使用事务性思维处理状态更新
- 错误发生时进行适当的状态回滚