use axum::{
    extract::Form,
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use redis::AsyncCommands;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::mysql;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct Service {
    mysql_pool: mysql::MySqlPool,
    redis_pool: redis::Client,
    http_client: reqwest::Client,
}

impl Service {
    async fn new() -> Self {
        Service {
            mysql_pool: mysql::MySqlPoolOptions::new()
                .max_connections(5)
                .connect("mysql://root:123456@localhost/test")
                .await
                .unwrap(),
            redis_pool: redis::Client::open("redis://127.0.0.1/").unwrap(),
            http_client: reqwest::Client::new(),
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_form=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let shared_state = Arc::new(Service::new().await);

    let app = Router::new()
        .route("/list", get(list))
        .route("/add", post(add))
        .with_state(shared_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::new(5, 0))),
        );

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn list(State(state): State<Arc<Service>>) -> Json<Value> {
    let row: (i64,) = sqlx::query_as("SELECT id from test")
        .bind(150_i64)
        .fetch_one(&state.mysql_pool)
        .await
        .unwrap();

    let mut con = state.redis_pool.get_async_connection().await.unwrap();

    let _: () = con.set("key1", b"foo").await.unwrap();

    let _: () = redis::cmd("SET")
        .arg(&["key2", "bar"])
        .query_async(&mut con)
        .await
        .unwrap();

    let result = redis::cmd("MGET")
        .arg(&["key1", "key2"])
        .query_async(&mut con)
        .await;
    let content = state
        .http_client
        .get("http://haokan.baidu.com/?_format=json")
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(result, Ok(("foo".to_string(), b"bar".to_vec())));
    dbg!(&row);
    Json(json!({ "id": row, "content": content}))
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Input {
    name: String,
    email: String,
}

async fn add(Form(input): Form<Input>) {
    dbg!(&input);
}


mod block {
    use std::time::SystemTime;
    use sha2::{Sha256, Digest};
    struct Block {
        prev_block_hash: Vec<u8>,
        data: Vec<u8>,
        timestamp: i64,
        hash: Vec<u8>,
    }
    
    impl Block {
        fn new(data: &str, prev_block_hash: &[u8]) -> Self {
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
            let block = Block {
                prev_block_hash: prev_block_hash.to_vec(),
                data: data.as_bytes().to_vec(),
                timestamp,
                hash: vec![],
            };
            
            block.set_hash();
            block_clone
        }
    
        fn SetHash(&mut self){
            let timestamp = self.timestamp.to_string().into_bytes();
            let headers = [self.prev_block_hash.as_slice(), self.data.as_slice(), &timestamp[..]].concat();
            let mut hasher = Sha256::new();
            hasher.update(headers);
            let hash = hasher.finalize();
            self.hash = hash.to_vec();
        }
    }
    }