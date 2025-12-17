#!/bin/bash

# 测试端口被占用时的错误处理
PORT=$1
MAXCONNS=5

# 首先启动一个进程占用端口
nc -l 127.0.0.1 $PORT &
NC_PID=$!

# 等待nc启动
sleep 1

# 尝试启动服务器，应该失败
echo "尝试在已占用的端口上启动服务器..."
./target/debug/ratsserver $MAXCONNS $PORT > output.txt 2>&1 &
SERVER_PID=$!

# 等待服务器尝试启动
sleep 1

# 检查服务器是否还在运行
if ps -p $SERVER_PID > /dev/null; then
    echo "❌ 测试失败：服务器在已占用的端口上启动成功"
    kill $SERVER_PID
    SUCCESS=1
else
    echo "✅ 测试通过：服务器在端口被占用时失败"
    SUCCESS=0
fi

# 杀死nc进程
kill $NC_PID

# 清理输出文件
rm output.txt

exit $SUCCESS