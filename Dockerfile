# ----------------------------------------------------------------------
# 阶段 1：编译阶段 (Builder Stage) - 使用 Musl 静态链接
# ----------------------------------------------------------------------
FROM rust:latest AS builder

# 安装 Musl 工具链和 C 语言级别的开发包
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev build-essential \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# 添加 Musl 编译目标
RUN rustup target add x86_64-unknown-linux-musl

# 设置容器工作目录为 /app
WORKDIR /app

# 1. 复制整个构建上下文 (即 /Users/tinachan/solji/ 目录下的所有内容)
COPY . .

# 2. 切换到索引器主项目目录
WORKDIR /app/solji-indexer 

# 3. 编译项目 - 关键：指定 musl 目标
# Cargo.toml 中可能需要指定 musl 静态链接，如果需要请手动调整
RUN cargo build --release --target x86_64-unknown-linux-musl

# ----------------------------------------------------------------------
# 阶段 2：运行阶段 (Runtime Stage) - 保持极小化镜像
# ----------------------------------------------------------------------
# 使用最小的 Alpine Linux 镜像 (它原生使用 Musl) 或 debian-slim 都可以，这里沿用 debian
FROM debian:bookworm-slim

# 安装运行时的 OpenSSL 库和 CA 证书 (用于 HTTPS/MySQL 连接)
RUN apt-get update && apt-get install -y \
    openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 复制编译好的二进制文件
# 重点：二进制文件现在位于 musl 目录
COPY --from=builder /app/solji-indexer/target/x86_64-unknown-linux-musl/release/solji-indexer /usr/local/bin/solji-indexer

# 暴露应用端口
EXPOSE 8080

# 定义容器启动时执行的命令
CMD ["/usr/local/bin/solji-indexer"]