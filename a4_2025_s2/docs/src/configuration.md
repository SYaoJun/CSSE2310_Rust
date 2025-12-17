# 配置与使用

本章节介绍如何配置和运行TCP服务器，包括安装依赖、编译和启动服务等步骤。

## 环境要求

- Rust 1.56.0 或更高版本
- Cargo 包管理器
- 支持Unix socket的操作系统（Linux/macOS）

## 安装依赖

首先确保已安装Rust环境：

```bash
# 检查Rust版本
rustc --version
cargo --version
```

如果未安装Rust，可以使用rustup安装：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 编译项目

克隆仓库后，进入项目目录编译：

```bash
# 克隆仓库（如果需要）
git clone <repository-url>
cd rust_a4

# 编译服务器\mcargo build --release --bin ratsserver

# 编译客户端（如果需要）\mcargo build --release --bin ratsclient
```

## 运行服务器

### 基本运行

使用默认端口（8888）运行服务器：

```bash
cargo run --bin ratsserver
```

### 指定端口

可以通过命令行参数指定服务器端口：

```bash
cargo run --bin ratsserver -- 9000
```

### 后台运行

在生产环境中，可以使用nohup或screen在后台运行服务器：

```bash
nohup cargo run --release --bin ratsserver > server.log 2>&1 &
```

## 配置选项

服务器支持以下配置方式：

### 命令行参数

- 端口号：作为第一个位置参数

### 环境变量

目前服务器不直接支持环境变量配置，但可以通过修改源码中的常量来调整以下参数：

- `DEFAULT_PORT`: 默认监听端口（8888）
- `MAX_PLAYER_NUM`: 最大玩家数量

## 连接测试

可以使用telnet或nc测试服务器连接：

```bash
# 使用telnet连接
telnet localhost 8888

# 或使用nc
nc localhost 8888
```

## 停止服务器

可以使用以下方法停止服务器：

- 在终端中按 Ctrl+C
- 找到进程ID并使用kill命令：
  ```bash
  ps aux | grep ratsserver
  kill <pid>
  ```