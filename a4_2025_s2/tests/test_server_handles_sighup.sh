#!/bin/bash

# 测试服务器是否能正确处理SIGHUP信号
PORT=$1
MAXCONNS=5

# 启动服务器
./target/debug/ratsserver $MAXCONNS $PORT > server_output.txt 2>&1 &
SERVER_PID=$!

# 等待服务器启动
sleep 1

# 检查服务器是否正在运行
if ! ps -p $SERVER_PID > /dev/null; then
    echo "❌ 测试失败：服务器未能启动"
    rm server_output.txt
    exit 1
fi

# 发送SIGHUP信号
echo "发送SIGHUP信号到服务器..."
kill -HUP $SERVER_PID

# 等待服务器处理信号
sleep 1

# 检查服务器输出中是否包含统计信息
if grep -q "Players connected:" server_output.txt; then
    echo "✅ 测试通过：服务器正确处理了SIGHUP信号"
    SUCCESS=0
else
    echo "❌ 测试失败：服务器没有响应SIGHUP信号"
    SUCCESS=1
fi

# 再次检查服务器是否仍然运行
if ! ps -p $SERVER_PID > /dev/null; then
    echo "❌ 测试失败：服务器在处理SIGHUP信号后崩溃"
    SUCCESS=1
fi

# 尝试连接服务器，验证它仍然可以接受连接
echo "验证服务器在处理SIGHUP后仍能接受连接..."
if nc -z 127.0.0.1 $PORT; then
    echo "✅ 测试通过：服务器在处理SIGHUP后仍能接受连接"
else
    echo "❌ 测试失败：服务器在处理SIGHUP后无法接受连接"
    SUCCESS=1
fi

# 杀死服务器
kill $SERVER_PID

# 清理输出文件
rm server_output.txt

exit $SUCCESS