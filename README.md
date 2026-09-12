# Simple Chat Room Server
- Using **Rust**
- set OS env `LOCAL_SERVER`

### shell
```
docker run -d --name [NAME] -p [PORT]:[PORT] -e LOCAL_SERBER="0.0.0.0:[PORT] --restart unless-stopped chat-room-server

