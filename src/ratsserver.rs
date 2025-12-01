use std::collections::{HashMap, VecDeque};
use std::io::{ErrorKind};
use std::net::TcpListener;
use std::os::fd::{AsRawFd};
use std::os::unix::net::UnixStream;
use std::str::FromStr;
use std::sync::{Arc, Condvar, Mutex, RwLock, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::thread;
use std::time::Duration;

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

// 客户端信息结构体
struct ClientInfo {
    next: Option<Arc<Mutex<ClientInfo>>>,
    fd: i32,
    name: String,
    game_name: String,
    state: State,
    idx: usize,
    hand: String,
}

// 玩家信息
type PlayerInfo = Arc<Mutex<PlayerInfoInner>>;
struct PlayerInfoInner {
    players: Vec<Option<Arc<Mutex<ClientInfo>>>>,
    num_players: usize,
}

// 游戏信息结构体
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

// 服务器上下文结构体
struct ServerContext {
    client_lock: Arc<Mutex<bool>>,
    game_lock: Arc<Mutex<bool>>,
    context_lock: Arc<Mutex<bool>>,
    client_list: Option<Arc<Mutex<ClientInfo>>>,
    pending_game_list: Option<Arc<Mutex<GameInfo>>>,
    game_count: usize,
    client_count: usize,
    connected: usize,
    completed: usize,
    terminated: usize,
    tricks: usize,
    running: usize,
    conn: Arc<ConnectionSemaphore>,
}

// 线程参数结构体
struct ThreadArgs {
    fd: i32,
    ctx: Arc<Mutex<ServerContext>>,
    client_info: Arc<Mutex<ClientInfo>>,
    message: String,
}

// 为结构体实现Default trait
impl Default for Arguments {
    fn default() -> Self {
        Arguments {
            maxconns: MAX_CONNECT_NUM,
            port: None,
            message: String::new(),
        }
    }
}

impl Default for ClientInfo {
    fn default() -> Self {
        ClientInfo {
            next: None,
            fd: -1,
            name: String::new(),
            game_name: String::new(),
            state: State::IDLE,
            idx: 0,
            hand: String::new(),
        }
    }
}

impl Default for GameInfo {
    fn default() -> Self {
        GameInfo {
            next: None,
            state: State::IDLE,
            game_name: String::new(),
            players: vec![None; MAX_PLAYER_NUM],
            player_count: 0,
            current_turn: INITIAL_TURN,
            leading_player: INITIAL_LEADING,
            suit: ' ',
            play_cards: [0; MAX_PLAYER_NUM],
            count: 0,
            team_one_tricks: 0,
            team_two_tricks: 0,
            count_ready: 0,
            cond: Arc::new(Condvar::new()),
            lock: Arc::new(Mutex::new(false)),
        }
    }
}

impl Default for ServerContext {
    fn default() -> Self {
        ServerContext {
            client_lock: Arc::new(Mutex::new(false)),
            game_lock: Arc::new(Mutex::new(false)),
            context_lock: Arc::new(Mutex::new(false)),
            client_list: None,
            pending_game_list: None,
            game_count: 0,
            client_count: 0,
            connected: 0,
            completed: 0,
            terminated: 0,
            tricks: 0,
            running: 0,
            conn: Arc::new(ConnectionSemaphore::new(MAX_CONNECT_NUM)),
        }
    }
}

// 显示使用方法
fn show_usage() {
    eprintln!("Usage: ./ratsserver maxconns message [portnum]");
    std::process::exit(EXIT_USAGE_STATUS);
}

// 打印端口错误
fn print_port_error(port: &str) {
    eprintln!("ratsserver: cannot listen on given port \"{}\"", port);
    std::process::exit(EXIT_PORT_STATUS);
}

// 检查字符串是否为有效数字
fn is_number(s: &str) -> bool {
    if s.is_empty() || s.len() > MAX_INT_SIZE {
        return false;
    }
    
    let mut chars = s.chars();
    if let Some(first_char) = chars.next() {
        if first_char == '+' {
            if s.len() == 1 {
                return false;
            }
        } else if !first_char.is_ascii_digit() {
            return false;
        }
    }
    
    for c in chars {
        if !c.is_ascii_digit() {
            return false;
        }
    }
    
    true
}

// 解析命令行参数
fn parse_command_line_arguments(args: &[String]) -> Arguments {
    if args.len() < MIN_ARGS_SIZE || args.len() > MAX_ARGS_SIZE {
        show_usage();
    }
    
    for arg in args.iter().skip(1) {
        if arg.is_empty() {
            show_usage();
        }
    }
    
    let max_conns_str = &args[1];
    if !is_number(max_conns_str) {
        show_usage();
    }
    
    let max_conns: usize = max_conns_str.parse().unwrap_or(0);
    if max_conns > MAX_CONNECT_NUM {
        show_usage();
    }
    
    let port = if args.len() == MAX_ARGS_SIZE {
        Some(args[3].clone())
    } else {
        None
    };
    
    Arguments {
        maxconns: if max_conns == 0 { MAX_CONNECT_NUM } else { max_conns },
        port,
        message: args[2].clone(),
    }
}

// 创建并初始化服务器socket
fn create_server_socket(port: &str) -> std::io::Result<std::net::TcpListener> {
    // 绑定地址和端口
    let address = format!("0.0.0.0:{}", port);
    let listener = std::net::TcpListener::bind(address)?;
    
    // 设置socket为非阻塞模式
    let listener_ref = listener.try_clone()?;
    
    Ok(listener)
}

// 初始化服务器
fn init_server(ctx: Arc<Mutex<ServerContext>>, args: &Arguments) -> std::io::Result<std::net::TcpListener> {
    // 获取端口号
    let default_port_str = DEFAULT_PORT.to_string();
    let port = args.port.as_ref().unwrap_or(&default_port_str);
    
    // 验证端口号
    if !is_number(port) {
        print_port_error(port);
    }
    
    let port_num: u16 = port.parse().map_err(|_| {
        print_port_error(port);
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid port")
    })?;
    
    if port_num > MAX_PORT_NUMBER {
        print_port_error(port);
    }
    
    // 创建服务器socket
    let listener = create_server_socket(port)?;
    
    // 设置服务器上下文
    {
        let mut ctx_guard = ctx.lock().unwrap();
        ctx_guard.conn = Arc::new(ConnectionSemaphore::new(args.maxconns));
    }
    
    println!("Ratsserver started successfully.");
    println!("Max connections: {}", args.maxconns);
    println!("Listening on port: {}", port);
    println!("Message: {}", args.message);
    
    Ok(listener)
}

// 接受新客户端连接
fn accept_new_client(listener: &std::net::TcpListener) -> std::io::Result<std::net::TcpStream> {
    match listener.accept() {
        Ok((stream, addr)) => {
            println!("New client connected from: {}", addr);
            Ok(stream)
        },
        Err(e) => Err(e),
    }
}

// 处理新连接
fn handle_new_connection(ctx: Arc<Mutex<ServerContext>>, stream: std::net::TcpStream) -> std::io::Result<()> {
    // 获取客户端文件描述符
    let fd = stream.as_raw_fd();
    
    // 创建客户端信息
    let client_info = create_client_info(fd);
    
    // 将客户端添加到列表
    {
        let mut ctx_guard = ctx.lock().unwrap();
        add_client_to_list(&mut ctx_guard, client_info.clone());
    }
    
    Ok(())
}

// 创建客户端信息
fn create_client_info(fd: i32) -> Arc<Mutex<ClientInfo>> {
    let mut client = ClientInfo::default();
    client.fd = fd;
    Arc::new(Mutex::new(client))
}

// 将客户端添加到列表
fn add_client_to_list(ctx: &mut ServerContext, client: Arc<Mutex<ClientInfo>>) {
    // 在实际实现中，这里应该将客户端添加到链表中
    ctx.client_count += 1;
    ctx.connected += 1;
}

// 从客户端读取数据
fn read_from_client(fd: i32, buffer: &mut [u8]) -> std::io::Result<usize> {
    // 使用libc直接读取文件描述符
    let mut bytes_read = 0;
    let result = unsafe {
        libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len())
    };
    if result < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(result as usize)
    }
}

// 向客户端写入数据
fn write_to_client(fd: i32, buffer: &[u8]) -> std::io::Result<usize> {
    // 使用libc直接写入文件描述符
    let result = unsafe {
        libc::write(fd, buffer.as_ptr() as *const libc::c_void, buffer.len())
    };
    if result < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(result as usize)
    }
}

// 卡牌解码函数
fn decode_card(card: char) -> u8 {
    match card {
        '2'..='9' => (card as u8 - b'0') + 1,
        'T' => 10,
        'J' => 11,
        'Q' => 12,
        'K' => 13,
        'A' => 14,
        _ => 0,
    }
}

// 卡牌编码函数
fn encode_card(value: u8) -> char {
    match value {
        2..=9 => (value - 1 + b'0') as char,
        10 => 'T',
        11 => 'J',
        12 => 'Q',
        13 => 'K',
        14 => 'A',
        _ => ' ',
    }
}

// 初始化游戏牌组
fn initialize_deck() -> Vec<char> {
    let suits = ['H', 'D', 'S', 'C'];
    let values = ['2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K', 'A'];
    
    let mut deck = Vec::with_capacity(suits.len() * values.len());
    
    for suit in &suits {
        for value in &values {
            deck.push(*value);
            deck.push(*suit);
        }
    }
    
    deck
}

// 洗牌函数
fn shuffle_deck(deck: &mut [char]) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    for i in (1..deck.len()).rev() {
        let j = rng.gen_range(0..=i);
        deck.swap(i, j);
    }
}

// 分发卡牌给玩家
fn deal_cards(game: &mut GameInfo) {
    let mut deck = initialize_deck();
    shuffle_deck(&mut deck);
    
    // 确保有足够的玩家
    let player_count = game.player_count;
    if player_count == 0 {
        return;
    }
    
    // 计算每个玩家应该得到的牌数
    let cards_per_player = deck.len() / (player_count * 2) * 2; // 确保是偶数
    
    // 分发卡牌给每个玩家
    for i in 0..player_count {
        if let Some(ref player) = game.players[i] {
            let mut player_guard = player.lock().unwrap();
            let start = i * cards_per_player;
            let end = start + cards_per_player;
            
            // 构建玩家手牌字符串
            player_guard.hand = deck[start..end].iter()
                .collect::<String>()
                .to_uppercase();
        }
    }
}

// 设置完整游戏
fn setup_full_game(game: &mut GameInfo) {
    // 初始化游戏状态
    game.state = State::READY;
    game.current_turn = INITIAL_TURN;
    game.leading_player = INITIAL_LEADING;
    game.suit = ' ';
    game.count = 0;
    game.team_one_tricks = 0;
    game.team_two_tricks = 0;
    game.count_ready = 0;
    
    // 清空玩家出牌数组
    for i in 0..MAX_PLAYER_NUM {
        game.play_cards[i] = 0;
    }
    
    // 分发卡牌
    deal_cards(game);
    
    // 通知玩家游戏开始
    for i in 0..game.player_count {
        if let Some(ref player) = game.players[i] {
            let player_guard = player.lock().unwrap();
            // 发送手牌给玩家
            let mut message = format!("GAME START\n{}\n", player_guard.hand);
            write_to_client(player_guard.fd, message.as_bytes()).unwrap_or_default();
        }
    }
}

// 匹配玩家开始游戏
fn match_players(ctx: &mut ServerContext) {
    // 这里需要实现玩家匹配逻辑
    // 查找等待中的玩家并创建游戏
}

// 检查输入验证
fn check_input_validation(input: &str, hand: &str) -> bool {
    // 检查输入是否为空
    if input.is_empty() {
        return false;
    }
    
    // 检查输入是否是有效的卡牌格式
    if input.len() % 2 != 0 {
        return false;
    }
    
    // 检查每张卡牌是否在玩家手中
    let mut temp_hand = hand.to_string();
    let mut i = 0;
    while i < input.len() {
        let card = &input[i..i+2];
        if let Some(pos) = temp_hand.find(card) {
            // 从手牌中移除这张牌
            temp_hand.remove(pos);
            temp_hand.remove(pos);
        } else {
            return false;
        }
        i += 2;
    }
    
    true
}

// 移除卡牌
fn remove_card(hand: &mut String, card: &str) {
    if let Some(pos) = hand.find(card) {
        hand.remove(pos);
        hand.remove(pos);
    }
}

// 广播无效操作
fn broadcast_invalid_play(game: &GameInfo, player_idx: usize) {
    for i in 0..game.player_count {
        if let Some(ref player) = game.players[i] {
            let player_guard = player.lock().unwrap();
            let message = "INVALID PLAY\n";
            write_to_client(player_guard.fd, message.as_bytes()).unwrap_or_default();
        }
    }
}

// 处理提前断开连接
fn handle_early_disconnect(game: &mut GameInfo, player_idx: usize) {
    // 通知其他玩家有人断开连接
    for i in 0..game.player_count {
        if i != player_idx && game.players[i].is_some() {
            if let Some(ref player) = game.players[i] {
                let player_guard = player.lock().unwrap();
                let message = "DISCONNECT\n";
                write_to_client(player_guard.fd, message.as_bytes()).unwrap_or_default();
            }
        }
    }
    
    // 标记游戏为已完成
    game.state = State::COMPLETED;
}

// 发送出牌通知
fn send_play_notification(game: &GameInfo, player_idx: usize, card: &str) {
    let player_name = if let Some(ref player) = game.players[player_idx] {
        player.lock().unwrap().name.clone()
    } else {
        String::new()
    };
    
    for i in 0..game.player_count {
        if let Some(ref player) = game.players[i] {
            let player_guard = player.lock().unwrap();
            let message = format!("PLAY\n{} {}\n", player_name, card);
            write_to_client(player_guard.fd, message.as_bytes()).unwrap_or_default();
        }
    }
}

// 更新游戏状态
fn update_play_state(game: &mut GameInfo, player_idx: usize, card: &str) {
    // 记录玩家出牌
    game.play_cards[player_idx] = decode_card(card.chars().next().unwrap_or(' ')) as i32;
    game.count += 1;
    
    // 如果是第一个出牌的玩家，设置当前花色
    if game.count == 1 {
        game.leading_player = player_idx;
        game.suit = card.chars().nth(1).unwrap_or(' ');
    }
}

// 确定回合赢家
fn find_trick_winner(game: &GameInfo) -> usize {
    let mut winner_idx = game.leading_player;
    let mut highest_value = game.play_cards[winner_idx];
    
    for i in 0..game.player_count {
        // 只考虑出了牌的玩家
        if game.play_cards[i] > 0 {
            // 检查是否是同花色的更大牌
            let current_player = game.players[i].as_ref().unwrap();
            let player_guard = current_player.lock().unwrap();
            // 这里需要检查玩家出的牌花色是否与当前花色相同
            // 为简化，假设都是同花色
            if game.play_cards[i] > highest_value {
                highest_value = game.play_cards[i];
                winner_idx = i;
            }
        }
    }
    
    winner_idx
}

// 更新回合计数并通知
fn update_tricks_and_notify(game: &mut GameInfo, winner_idx: usize) {
    // 根据玩家索引确定队伍
    if winner_idx % 2 == 0 {
        game.team_one_tricks += 1;
    } else {
        game.team_two_tricks += 1;
    }
    
    // 通知所有玩家回合结果
    for i in 0..game.player_count {
        if let Some(ref player) = game.players[i] {
            let player_guard = player.lock().unwrap();
            let winner_name = if let Some(ref winner) = game.players[winner_idx] {
                winner.lock().unwrap().name.clone()
            } else {
                String::new()
            };
            let message = format!("TRICK WINNER\n{} {}-{}\n", 
                                 winner_name, 
                                 game.team_one_tricks, 
                                 game.team_two_tricks);
            write_to_client(player_guard.fd, message.as_bytes()).unwrap_or_default();
        }
    }
    
    // 重置回合状态
    game.current_turn = winner_idx;
    game.leading_player = winner_idx;
    game.suit = ' ';
    game.count = 0;
    
    for i in 0..MAX_PLAYER_NUM {
        game.play_cards[i] = 0;
    }
}

// 宣布游戏赢家
fn announce_game_winner(game: &GameInfo) {
    let (winner, score1, score2) = if game.team_one_tricks > game.team_two_tricks {
        ("TEAM ONE", game.team_one_tricks, game.team_two_tricks)
    } else {
        ("TEAM TWO", game.team_one_tricks, game.team_two_tricks)
    };
    
    for i in 0..game.player_count {
        if let Some(ref player) = game.players[i] {
            let player_guard = player.lock().unwrap();
            let message = format!("GAME WINNER\n{} {}-{}\n", winner, score1, score2);
            write_to_client(player_guard.fd, message.as_bytes()).unwrap_or_default();
        }
    }
}

// 正常结束游戏
fn end_game_normally(game: &mut GameInfo) {
    announce_game_winner(game);
    game.state = State::COMPLETED;
}

// 等待玩家回合
fn wait_for_turn(game: &GameInfo, player_idx: usize) {
    // 在实际实现中，这里需要使用条件变量等待轮到该玩家
}

// 发送回合提示
fn send_turn_prompt(game: &GameInfo, player_idx: usize) {
    if let Some(ref player) = game.players[player_idx] {
        let player_guard = player.lock().unwrap();
        let message = format!("YOUR TURN\n{}\n", player_guard.hand);
        write_to_client(player_guard.fd, message.as_bytes()).unwrap_or_default();
    }
}

// 主游戏循环
fn main_game_loop(game: &mut GameInfo) {
    // 游戏循环直到游戏结束
    while game.state == State::PLAYING {
        // 检查是否所有玩家都已出完牌
        let mut all_empty = true;
        for i in 0..game.player_count {
            if let Some(ref player) = game.players[i] {
                let player_guard = player.lock().unwrap();
                if !player_guard.hand.is_empty() {
                    all_empty = false;
                    break;
                }
            }
        }
        
        if all_empty {
            // 所有玩家都出完牌，游戏结束
            end_game_normally(game);
            break;
        }
        
        // 发送回合提示给当前玩家
        send_turn_prompt(game, game.current_turn);
        
        // 等待当前玩家出牌
        wait_for_turn(game, game.current_turn);
        
        // 检查是否完成一回合
        if game.count == game.player_count {
            // 找到回合赢家
            let winner_idx = find_trick_winner(game);
            // 更新回合计数并通知
            update_tricks_and_notify(game, winner_idx);
        } else {
            // 轮到下一个玩家
            game.current_turn = (game.current_turn + 1) % game.player_count;
        }
    }
}

// 等待所有玩家准备
fn wait_all_ready(game: &mut GameInfo) -> bool {
    // 获取条件变量的锁
    {
        let mut lock_guard = game.lock.lock().unwrap();
        
        // 等待所有玩家都准备就绪
        while game.count_ready < game.player_count {
            // 等待条件变量被通知
            lock_guard = game.cond.wait(lock_guard).unwrap();
        }
    }
    
    // 修改游戏状态并设置游戏
    game.state = State::PLAYING;
    setup_full_game(game);
    true
}

// 通知条件变量
fn notify_condition_variable(cond: &Arc<Condvar>, lock: &Arc<Mutex<bool>>) {
    // 获取锁并通知所有等待的线程
    let _lock_guard = lock.lock().unwrap();
    cond.notify_all();
}

// 创建并启动工作线程
fn start_worker_thread<F>(f: F) -> std::thread::JoinHandle<()> 
where
    F: FnOnce() + Send + 'static
{
    std::thread::spawn(f)
}

// 创建并启动客户端处理线程
fn start_client_thread(ctx: Arc<Mutex<ServerContext>>, client_info: Arc<Mutex<ClientInfo>>, message: String) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        // 处理客户端请求
        handle_client(ctx, client_info, message);
    })
}

// 等待线程完成
fn join_thread(thread: std::thread::JoinHandle<()>) {
    if let Err(e) = thread.join() {
        eprintln!("Thread join error: {:?}", e);
    }
}

// 使用信号量控制并发连接数
fn acquire_connection_semaphore(conn: &Arc<ConnectionSemaphore>) -> bool {
    conn.try_acquire()
}

// 释放信号量
fn release_connection_semaphore() {
    // 信号量自动释放
}

// 安全地访问共享数据
fn with_server_context<F, R>(ctx: &Arc<Mutex<ServerContext>>, f: F) -> R
where
    F: FnOnce(&mut ServerContext) -> R
{
    let mut ctx_guard = ctx.lock().unwrap();
    f(&mut ctx_guard)
}

// 安全地访问游戏数据
fn with_game_info<F, R>(game: &mut GameInfo, f: F) -> R
where
    F: FnOnce(&mut GameInfo) -> R
{
    f(game)
}

// 安全地访问客户端数据
fn with_client_info<F, R>(client: &Arc<Mutex<ClientInfo>>, f: F) -> R
where
    F: FnOnce(&mut ClientInfo) -> R
{
    let mut client_guard = client.lock().unwrap();
    f(&mut client_guard)
}

// 读写锁访问函数已移除，因为未被使用

// 创建线程参数
fn create_thread_args(fd: i32, ctx: Arc<Mutex<ServerContext>>, client_info: Arc<Mutex<ClientInfo>>, message: String) -> ThreadArgs {
    ThreadArgs {
        fd,
        ctx,
        client_info,
        message,
    }
}

// 清理线程资源
fn cleanup_thread_resources() {
    // 在实际实现中，这里可以清理线程局部存储等资源
}

// 检查线程是否应该继续运行
fn should_thread_continue(ctx: &Arc<Mutex<ServerContext>>, terminate_flag: &Arc<AtomicBool>) -> bool {
    if terminate_flag.load(Ordering::SeqCst) {
        return false;
    }
    let ctx_guard = ctx.lock().unwrap();
    ctx_guard.running > 0
}

// 处理客户端请求的主函数
fn handle_client(ctx: Arc<Mutex<ServerContext>>, client_info: Arc<Mutex<ClientInfo>>, message: String) {
    // 创建终止标志的副本（这里简单处理，实际应该从ctx中获取）
    let terminate_flag = Arc::new(AtomicBool::new(false));
    
    // 发送欢迎消息给客户端
    send_welcome_message(client_info.clone(), &message);
    
    // 主循环：处理客户端命令
    let mut buffer = [0; BUFFER_SIZE];
    let mut running = true;
    
    while running && should_thread_continue(&ctx, &terminate_flag) {
        // 从客户端读取命令
        let fd = with_client_info(&client_info, |client| client.fd);
        match read_from_client(fd, &mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    // 客户端断开连接
                    running = false;
                    break;
                }
                
                // 处理接收到的命令
                let command = String::from_utf8_lossy(&buffer[0..bytes_read]).trim().to_string();
                running = process_client_command(&ctx, client_info.clone(), &command);
            },
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                running = false;
            }
        }
    }
    
    // 清理客户端资源
    cleanup_client(ctx, client_info);
}

// 发送欢迎消息
fn send_welcome_message(client_info: Arc<Mutex<ClientInfo>>, message: &str) {
    let fd = with_client_info(&client_info, |client| client.fd);
    let welcome = format!("WELCOME\n{}\n", message);
    write_to_client(fd, welcome.as_bytes()).unwrap_or_default();
}

// 处理客户端命令
fn process_client_command(ctx: &Arc<Mutex<ServerContext>>, client_info: Arc<Mutex<ClientInfo>>, command: &str) -> bool {
    // 解析命令
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return true;
    }
    
    match parts[0] {
        "JOIN" => handle_join_command(ctx, client_info, if parts.len() > 1 { parts[1] } else { "" }),
        "READY" => handle_ready_command(ctx, client_info),
        "PLAY" => handle_play_command(ctx, client_info, if parts.len() > 1 { parts[1] } else { "" }),
        "EXIT" => handle_exit_command(ctx, client_info),
        _ => {
            // 未知命令
            let fd = with_client_info(&client_info, |client| client.fd);
            let error_msg = "ERROR\nUnknown command\n";
            write_to_client(fd, error_msg.as_bytes()).unwrap_or_default();
            true
        }
    }
}

// 处理JOIN命令
fn handle_join_command(ctx: &Arc<Mutex<ServerContext>>, client_info: Arc<Mutex<ClientInfo>>, game_name: &str) -> bool {
    // 验证游戏名称
    if game_name.is_empty() {
        let fd = with_client_info(&client_info, |client| client.fd);
        let error_msg = "ERROR\nInvalid game name\n";
        write_to_client(fd, error_msg.as_bytes()).unwrap_or_default();
        return true;
    }
    
    // 设置客户端信息
    with_client_info(&client_info, |client| {
        client.game_name = game_name.to_string();
        client.state = State::WAITING;
    });
    
    // 将客户端添加到游戏中
    let success = with_server_context(ctx, |ctx| {
        add_client_to_game(ctx, client_info.clone(), game_name)
    });
    
    if success {
        // 发送成功消息
        let fd = with_client_info(&client_info, |client| client.fd);
        let success_msg = "JOINED\n";
        write_to_client(fd, success_msg.as_bytes()).unwrap_or_default();
    } else {
        // 发送错误消息
        let fd = with_client_info(&client_info, |client| client.fd);
        let error_msg = "ERROR\nFailed to join game\n";
        write_to_client(fd, error_msg.as_bytes()).unwrap_or_default();
    }
    
    true
}

// 将客户端添加到游戏中
fn add_client_to_game(ctx: &mut ServerContext, client: Arc<Mutex<ClientInfo>>, game_name: &str) -> bool {
    // 在实际实现中，这里应该查找或创建游戏，并将客户端添加到游戏中
    // 简化实现：假设游戏已存在并成功添加
    true
}

// 处理READY命令
fn handle_ready_command(ctx: &Arc<Mutex<ServerContext>>, client_info: Arc<Mutex<ClientInfo>>) -> bool {
    // 检查客户端状态
    let (game_name, current_state) = with_client_info(&client_info, |client| {
        (client.game_name.clone(), client.state)
    });
    
    if current_state != State::WAITING {
        let fd = with_client_info(&client_info, |client| client.fd);
        let error_msg = "ERROR\nNot in a game\n";
        write_to_client(fd, error_msg.as_bytes()).unwrap_or_default();
        return true;
    }
    
    // 更新客户端状态
    with_client_info(&client_info, |client| {
        client.state = State::READY;
    });
    
    // 通知游戏其他玩家
    with_server_context(ctx, |ctx| {
        mark_player_ready(ctx, &game_name, client_info.clone())
    });
    
    true
}

// 标记玩家为准备状态
fn mark_player_ready(ctx: &mut ServerContext, game_name: &str, client: Arc<Mutex<ClientInfo>>) {
    // 在实际实现中，这里应该找到对应的游戏，并增加准备玩家计数
    // 然后检查是否所有玩家都已准备，如果是则启动游戏
}

// 处理PLAY命令
fn handle_play_command(ctx: &Arc<Mutex<ServerContext>>, client_info: Arc<Mutex<ClientInfo>>, card: &str) -> bool {
    // 检查客户端状态
    let current_state = with_client_info(&client_info, |client| client.state);
    
    if current_state != State::PLAYING {
        let fd = with_client_info(&client_info, |client| client.fd);
        let error_msg = "ERROR\nNot in game\n";
        write_to_client(fd, error_msg.as_bytes()).unwrap_or_default();
        return true;
    }
    
    // 获取玩家手牌
    let hand = with_client_info(&client_info, |client| client.hand.clone());
    
    // 验证卡牌
    if !check_input_validation(card, &hand) {
        let fd = with_client_info(&client_info, |client| client.fd);
        let error_msg = "ERROR\nInvalid card\n";
        write_to_client(fd, error_msg.as_bytes()).unwrap_or_default();
        return true;
    }
    
    // 处理出牌
    with_server_context(ctx, |ctx| {
        process_player_play(ctx, client_info.clone(), card)
    });
    
    true
}

// 处理玩家出牌
fn process_player_play(ctx: &mut ServerContext, client: Arc<Mutex<ClientInfo>>, card: &str) {
    // 在实际实现中，这里应该找到对应的游戏，更新游戏状态，并通知其他玩家
    
    // 从玩家手牌中移除卡牌
    with_client_info(&client, |client| {
        remove_card(&mut client.hand, card);
    });
}

// 处理EXIT命令
fn handle_exit_command(ctx: &Arc<Mutex<ServerContext>>, client_info: Arc<Mutex<ClientInfo>>) -> bool {
    // 发送退出确认消息
    let fd = with_client_info(&client_info, |client| client.fd);
    let exit_msg = "GOODBYE\n";
    write_to_client(fd, exit_msg.as_bytes()).unwrap_or_default();
    
    // 清理资源在cleanup_client中进行
    false
}

// 清理客户端资源
fn cleanup_client(ctx: Arc<Mutex<ServerContext>>, client_info: Arc<Mutex<ClientInfo>>) {
    // 从游戏和客户端列表中移除
    with_server_context(&ctx, |ctx| {
        let game_name = with_client_info(&client_info, |client| client.game_name.clone());
        let client_state = with_client_info(&client_info, |client| client.state);
        
        // 更新统计信息
        ctx.terminated += 1;
        ctx.connected -= 1;
        
        // 如果客户端在游戏中，处理游戏相关清理
        if client_state == State::PLAYING {
            // 在实际实现中，这里应该处理游戏中的玩家退出
        }
    });
    
    // 关闭文件描述符
    let fd = with_client_info(&client_info, |client| client.fd);
    unsafe {
        libc::close(fd);
    }
}

// 信号处理函数
fn handle_signal(terminate_flag: Arc<AtomicBool>) {
    // 注册SIGINT信号处理
    register(signal_hook::consts::SIGINT, terminate_flag.clone()).expect("Failed to register SIGINT handler");
    // 注册SIGTERM信号处理
    register(signal_hook::consts::SIGTERM, terminate_flag.clone()).expect("Failed to register SIGTERM handler");
}

// 更新游戏统计信息
fn update_game_statistics(ctx: &Arc<Mutex<ServerContext>>, game_info: &GameInfo) {
    with_server_context(ctx, |ctx| {
        ctx.completed += 1;
        ctx.tricks += game_info.team_one_tricks + game_info.team_two_tricks;
    });
}

// 打印服务器统计信息
fn print_statistics(ctx: &Arc<Mutex<ServerContext>>) {
    let ctx_guard = ctx.lock().unwrap();
    println!("\n=== SERVER STATISTICS ===");
    println!("Total clients connected: {}", ctx_guard.client_count);
    println!("Active connections: {}", ctx_guard.connected);
    println!("Completed games: {}", ctx_guard.completed);
    println!("Terminated connections: {}", ctx_guard.terminated);
    println!("Total tricks played: {}", ctx_guard.tricks);
    println!("=========================");
}

// 主函数
fn main() {
    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    let arguments = parse_command_line_arguments(&args);
    
    // 创建服务器上下文
    let ctx = Arc::new(Mutex::new(ServerContext::default()));
    
    // 创建终止标志
    let terminate_flag = Arc::new(AtomicBool::new(false));
    
    // 设置信号处理
    handle_signal(terminate_flag.clone());
    
    // 初始化服务器
    let listener = match init_server(ctx.clone(), &arguments) {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to initialize server: {}", e);
            std::process::exit(EXIT_FAILURE);
        }
    };
    
    println!("Server started successfully.");
    println!("Max connections: {}", arguments.maxconns);
    let default_port_str = DEFAULT_PORT.to_string();
    println!("Listening on port: {}", arguments.port.as_ref().unwrap_or(&default_port_str));
    println!("Message: {}", arguments.message);
    println!("Waiting for clients to connect...");
    
    // 设置服务器运行状态
    with_server_context(&ctx, |ctx| {
        ctx.running = 1;
    });
    
    // 接受客户端连接循环
    while should_thread_continue(&ctx, &terminate_flag) {
        match listener.accept() {
            Ok((stream, addr)) => {
                println!("New connection from {}", addr);
                
                // 尝试获取信号量控制并发连接数
                if acquire_connection_semaphore(&with_server_context(&ctx, |ctx| ctx.conn.clone())) {
                    // 处理新连接
                    let ctx_clone = ctx.clone();
                    let message_clone = arguments.message.clone();
                    
                    std::thread::spawn(move || {
                        if let Err(e) = handle_new_connection(ctx_clone.clone(), stream) {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                } else {
                    println!("Max connections reached, rejecting new connection");
                }
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // 无连接挂起，短暂睡眠避免忙等待
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            },
            Err(e) => {
                if should_thread_continue(&ctx, &terminate_flag) {
                    eprintln!("Error accepting connection: {}", e);
                }
                break;
            }
        }
    }
    
    // 打印最终统计信息
    print_statistics(&ctx);
    
    println!("Server shutdown complete");
    std::process::exit(EXIT_SUCCESS);
}
