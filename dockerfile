FROM rust:1.85-alpine
WORKDIR /app
RUN apk add musl-dev
COPY . .
#构建
RUN cargo build --release

#设置镜像构建时的环境变量
ARG VERSION=1.0.0

#设置维护者信息
LABEL maintainer="ln196800594@outlook.com"
LABEL description="A simple chat room using Rust"
LABEL version=$VERSION

#设置运行时的环境变量
ENV LOCAL_SERVER="0.0.0.0:10086"

CMD ["./target/release/server"]