#!/bin/bash

echo "开始安装 svn-hook..."

# 检查是否安装了 Rust
if ! command -v cargo &> /dev/null; then
    echo "错误: 未安装 Rust"
    echo "请先安装 Rust:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 执行安装
echo "正在编译并安装..."
if cargo install --path .; then
    echo "✅ 安装成功"
    
    # 清理 target 目录
    echo "正在清理编译文件..."
    if rm -rf target/; then
        echo "✅ 清理完成"
    else
        echo "警告: 清理 target 目录失败"
    fi
    
    echo -e "\n使用说明:"
    echo "1. 确保已创建配置文件: ~/.config/commit_crafter/config.toml"
    echo "2. 在 SVN 工作目录中执行: svn-hook commit"
else
    echo "❌ 安装失败"
    exit 1
fi 