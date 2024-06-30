use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use chat_core::{Chat, ChatType};
use chat_server::AppState;
use futures::StreamExt;
use notify_server::{get_router, AppConfig};
use reqwest::multipart::Part;
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, time::sleep};

const WILD_ADDR: &str = "0.0.0.0:0";
struct ChatServer {
    addr: SocketAddr,
    token: String,
    client: reqwest::Client,
}

struct NotifyServer;

#[tokio::test]
async fn chat_server_should_work() -> Result<()> {
    let (tdb, state) = AppState::new_for_test().await?;
    let chat_server = ChatServer::new(state).await?;
    let db_url = tdb.url();
    NotifyServer::new(&db_url, &chat_server.token).await?;

    let members = vec![1, 2, 3, 4, 5, 6];
    let chat = chat_server
        .create_chat("chat2", members.clone(), 4, true)
        .await?;
    assert_eq!(chat.name, Some("chat2".to_string()));
    assert_eq!(chat.members, members);
    assert_eq!(chat.ws_id, 4);
    assert_eq!(chat.r#type, ChatType::PublicChannel);

    chat_server.send_message(chat.id, "hello").await?;

    sleep(Duration::from_secs(1)).await;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct AuthToken {
    token: String,
}

impl ChatServer {
    pub async fn new(state: AppState) -> Result<Self> {
        let app = chat_server::get_router(state).await?;
        let lisitener = TcpListener::bind(WILD_ADDR)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let addr = lisitener.local_addr().map_err(|e| anyhow::anyhow!(e))?;
        tokio::spawn(async move {
            axum::serve(lisitener, app.into_make_service())
                .await
                .unwrap();
        });

        let client = reqwest::Client::new();
        let mut chat_server = Self {
            addr,
            token: "".to_string(),
            client,
        };
        let username = "jack";
        let email = "jack@none.org";
        let password = "123456";
        let workspace = "jack";
        chat_server
            .signup(username, email, password, workspace)
            .await?;
        let ws_id = 4;
        let token = chat_server.signin(email, password, ws_id).await?;
        chat_server.token = token;
        Ok(chat_server)
    }

    pub async fn signin(&mut self, email: &str, password: &str, ws_id: i64) -> Result<String> {
        let res = self
            .client
            .post(&format!("http://localhost:{}/api/signin", self.addr.port()))
            .json(&serde_json::json!({ "email": email, "password": password, "ws_id": ws_id }))
            .send()
            .await?;
        assert_eq!(res.status(), 200);
        let token = res.json::<AuthToken>().await?;
        Ok(token.token)
    }
    pub async fn signup(
        &self,
        fullname: &str,
        email: &str,
        password: &str,
        workspace: &str,
    ) -> Result<()> {
        let res = self
            .client
            .post(&format!("http://localhost:{}/api/signup", self.addr.port()))
            .json(&serde_json::json!({ "fullname": fullname, "email": email, "password": password, "workspace": workspace }))
            .send()
            .await?;
        assert_eq!(res.status(), 201);
        Ok(())
    }

    pub async fn create_chat(
        &self,
        name: &str,
        members: Vec<i64>,
        ws_id: i64,
        public: bool,
    ) -> Result<Chat> {
        let res = self
            .client
            .post(&format!("http://localhost:{}/api/chats", self.addr.port()))
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&serde_json::json!({ "name": name, "members": members, "ws_id": ws_id, "public": public }))
            .send()
            .await?;
        assert_eq!(res.status(), 201);
        let chat = res.json::<Chat>().await?;
        assert_eq!(chat.name, Some(name.to_string()));
        assert_eq!(chat.members, members);
        assert_eq!(chat.ws_id, ws_id);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        Ok(chat)
    }

    pub async fn send_message(&self, chat_id: i64, content: &str) -> Result<()> {
        let bytes = include_bytes!("../Cargo.toml");
        let part = Part::bytes(bytes.to_vec())
            .file_name("Cargo.toml")
            .mime_str("text/plain")
            .unwrap();

        let resp = self
            .client
            .post(&format!("http://localhost:{}/api/upload", self.addr.port()))
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(reqwest::multipart::Form::new().part("file", part))
            .send()
            .await?;
        assert_eq!(resp.status(), 200);
        let files = resp.json::<Vec<String>>().await?;

        let res = self
            .client
            .post(&format!(
                "http://localhost:{}/api/chats/{}",
                self.addr.port(),
                chat_id
            ))
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&serde_json::json!({ "content": content, "files": files }))
            .send()
            .await?;
        assert_eq!(res.status(), 201);
        Ok(())
    }
}

impl NotifyServer {
    pub async fn new(db_url: &str, token: &str) -> Result<Self> {
        let mut config = AppConfig::load()?;
        println!("db_url: {}", db_url);
        config.server.db_url = db_url.to_string();
        let app = get_router(config).await?;

        let addr = "0.0.0.0:0";
        let listener = TcpListener::bind(addr).await?;
        let addr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let mut es = EventSource::get(format!(
            "http://localhost:{}/events?token={}",
            addr.port(),
            token
        ));

        tokio::spawn(async move {
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => {
                        println!("connection opened");
                    }
                    Ok(Event::Message(msg)) => {
                        println!("message: {:?}", msg);
                    }
                    Err(e) => {
                        eprintln!("error: {:?}", e);
                        break;
                    }
                }
            }
        });
        sleep(Duration::from_secs(1)).await;
        Ok(Self)
    }
}
