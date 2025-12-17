#!/bin/bash

# 运行所有服务器测试脚本
PORT=8888

# 确保服务器已编译
cargo build --bin ratsserver

# 使测试脚本可执行
chmod +x tests/test_server_accepts_connections.sh
chmod +x tests/test_server_port_in_use.sh
chmod +x tests/test_server_handles_sighup.sh

# 运行测试
echo "运行测试：服务器接受连接"
tests/test_server_accepts_connections.sh $PORT
TEST1=$?

echo "\n运行测试：端口被占用"
tests/test_server_port_in_use.sh $PORT
TEST2=$?

echo "\n运行测试：SIGHUP信号处理"
tests/test_server_handles_sighup.sh $PORT
TEST3=$?

# 总结测试结果
echo "\n=========================="
echo "测试结果总结"
echo "=========================="

if [ $TEST1 -eq 0 ]; then
    echo "✅ 服务器接受连接：通过"
else
    echo "❌ 服务器接受连接：失败"
fi

if [ $TEST2 -eq 0 ]; then
    echo "✅ 端口被占用：通过"
else
    echo "❌ 端口被占用：失败"
fi

if [ $TEST3 -eq 0 ]; then
    echo "✅ SIGHUP信号处理：通过"
else
    echo "❌ SIGHUP信号处理：失败"
fi

# 检查是否所有测试都通过
if [ $TEST1 -eq 0 ] && [ $TEST2 -eq 0 ] && [ $TEST3 -eq 0 ]; then
    echo "\n🎉 所有测试都通过了！"
    exit 0
else
    echo "\n❌ 一些测试失败了。"
    exit 1
fi