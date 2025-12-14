use std::fs;
use log;
use env_logger;
/// 初始化日志系统，创建带时间戳的日志文件
pub fn init_logging() {
    // 创建log目录（如果不存在）
    fs::create_dir_all("log").expect("无法创建log目录");

    let the_time = chrono::Local::now()
        .format("%Y_%m_%d_%H:%M:%S")
        .to_string();


    // 构造日志文件路径
    let log_file_path = format!("log/uqentropy_{}.log", the_time);

    // 设置环境变量以配置env_logger
    std::env::set_var("RUST_LOG", "debug");

    // 初始化env_logger并设置输出到文件
    let log_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_file_path)
        .expect("无法创建日志文件");

    env_logger::builder()
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .format_timestamp_secs()
        .init();

    log::info!("日志系统已初始化，日志文件: {}", log_file_path);
}