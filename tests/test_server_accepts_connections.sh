#!/bin/bash

# 测试服务器是否能接受连接
PORT=$1
MAXCONNS=5

# 启动服务器
./target/debug/ratsserver $MAXCONNS $PORT &
SERVER_PID=$!

# 等待服务器启动
sleep 1

# 尝试连接服务器
echo "尝试连接到服务器..."
if nc -z 127.0.0.1 $PORT; then
    echo "✅ 测试通过：服务器接受连接"
    SUCCESS=0
else
    echo "❌ 测试失败：服务器拒绝连接"
    SUCCESS=1
fi

# 杀死服务器
kill $SERVER_PID

# 等待服务器关闭
sleep 1

exit $SUCCESS