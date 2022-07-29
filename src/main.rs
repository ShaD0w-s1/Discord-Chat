use std::{net::SocketAddr, sync::Arc};

/*
 * @Author: zhangyuxuan
 * @Date: 2022-07-11 21:44:05
 * @LastEditTime: 2022-07-18 18:26:44
 * @LastEditors: zhangyuxuan
 * @FilePath: \Discord-Chat\src\main.rs
 */
use axum::{
    body,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router, Server, TypedHeader, http::header,
};

use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    protocol::channel,
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties,
};

async fn handler(ws: WebSocketUpgrade,Extension(channel): Extension<Channel>) -> Response {
    ws.on_upgrade(|mut socket| async move {
        let queue = channel
            .queue_declare(
                "server_1",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();

        let a = channel
            .basic_consume(
                "server_1",
                "consumer_1",
                lapin::options::BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap()
            .set_delegate(move |message| async {
                let message = match message {
                    Ok(Some(message)) => message,
                    Ok(None) => return,
                    Err(e) => {
                        panic!("{}", e);
                    }
                };

                println!("{:?}", message);
            });

        // let a = match socket.recv().await {
        //     Some(Ok(b)) => ,
        //     Some(Err(e)) => {
        //         panic!("{}", e);
        //     }
        //     None => return,
        // };
    })
}

#[tokio::main]
async fn main() {
    let connection = Connection::connect(
        "amqp://guest:guest@127.0.0.1:5672",
        ConnectionProperties::default(),
    )
    .await
    .expect("MQ连接失败");

    let channel = connection.create_channel().await.expect("MQ创建通道失败");

    let app = Router::new()
        .route("/ws", get(handler))
        .layer(Extension(channel))
        .into_make_service();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    Server::bind(&addr)
        .serve(app)
        .await
        .expect("服务器启动失败");
}
