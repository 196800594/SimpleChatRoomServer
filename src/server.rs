use tokio::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use futures::channel::mpsc::{ self, UnboundedSender };
pub use futures::{future, StreamExt, TryFutureExt, TryStreamExt, SinkExt};
use tokio_tungstenite::tungstenite::Message;

type PeerMap = Arc<Mutex<HashMap<SocketAddr, UnboundedSender<Message>>>>;

#[derive(Debug)]
pub struct Server;

impl Server {
    pub async fn run(){
        let local_server = env::var("LOCAL_SERVER").unwrap_or_else(|_| "0.0.0.0:10086".to_string());
        let addr = local_server.clone();
        let listener = TcpListener::bind(local_server).await.expect("Server>> 主机名或端口异常");
        println!("Server>> 服务已启动在 {}", addr);

        let peers: PeerMap = Arc::new(Mutex::new(HashMap::new()));

        while let Ok((stream, peer_addr)) = listener.accept().await {
            let peers = Arc::clone(&peers);
            tokio::spawn(handle_connection(stream, peer_addr, peers));
        }


    }

}

async fn handle_connection(stream: TcpStream, peer_addr: SocketAddr, peers: PeerMap) {
    let ws_stream
        = tokio_tungstenite::accept_async(stream).await.expect("Server>> 握手失败，无法建立 WebSocket 连接");
    println!("Server>> 来自 {} 的连接", peer_addr);

    let (msg_sender, msg_receiver) = mpsc::unbounded();
    let (write, read) = ws_stream.split();

    //把新的客户端加到peers中
    peers.lock().unwrap().insert(peer_addr, msg_sender);

    let receive_and_broadcast_messages = read.try_for_each(|message: Message| {
        match message.clone() {
            Message::Text(text_message) => {
                println!("Server>> 来自 {} 的消息: {}", peer_addr, text_message);
                //向其他客户端广播收到的消息
                let peers = peers.lock().unwrap();
                let broadcast_peers = peers.iter().filter(|(addr, _)| **addr != peer_addr);
                for (broadcast_addr, broadcast_peer) in broadcast_peers {
                    if !broadcast_peer.is_closed() {
                        if let Err(err) = broadcast_peer.unbounded_send(message.clone()){
                            eprintln!("Server>> 无法广播消息 {:?} ({})", err, broadcast_addr)
                        }
                    }
                }
                future::ok(())
            }
            Message::Close(_) => {
                //客户端断开连接
                peers.lock().unwrap().remove(&peer_addr);
                println!("Server>> {} 断开了连接", peer_addr);
                future::err(tokio_tungstenite::tungstenite::Error::ConnectionClosed)
            }
            _ => {
                println!("Server>> 不支持的消息格式");
                future::ok(())
            }
        }
    });


    let forward_messages = msg_receiver.map(Ok).forward(write);

    if let Err(err) = tokio::try_join!(receive_and_broadcast_messages, forward_messages) {
        eprintln!("Server>> 广播时异常 {:?}", err);

        //非正常断开连接，需要清理peers信息
        peers.lock().unwrap().remove(&peer_addr);
        println!("Server>> {} 断开连接", peer_addr);
    }

}
