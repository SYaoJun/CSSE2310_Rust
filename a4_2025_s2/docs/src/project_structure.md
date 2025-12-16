# 项目结构

## 目录结构

本项目采用清晰的分层结构，主要源代码位于`src`目录下，测试和示例文件位于`testfiles`目录。

```
├── src/
│   ├── main.rs        # 主入口文件
│   ├── ratsserver.rs  # 服务器核心实现
│   └── ratsclient.rs  # 客户端实现
├── testfiles/         # 测试相关文件和脚本
├── Cargo.toml         # Rust项目配置文件
└── Makefile           # 构建脚本
```

## 核心文件说明

### main.rs

主入口文件，负责解析命令行参数、初始化服务器并启动主循环。

### ratsserver.rs

服务器核心实现，包含以下主要组件：

- **数据结构定义**: ClientInfo、GameInfo、ServerContext等
- **服务器初始化**: init_server函数
- **连接处理**: handle_accept等函数
- **游戏逻辑**: setup_full_game、wait_all_ready等函数
- **多线程管理**: start_worker_thread等函数

### ratsclient.rs

客户端实现，负责与服务器建立连接并发送命令。

## 模块划分

服务器代码可以分为以下几个主要模块：

1. **数据结构层**: 定义所有核心数据结构
2. **网络层**: 处理TCP连接和数据传输
3. **业务逻辑层**: 实现游戏规则和状态管理
4. **线程管理层**: 处理多线程并发和同步