use axum::{
    extract::Form,
    response::Json,
    routing::{get, post},
    Router,
    extract::State,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::mysql;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use redis::AsyncCommands;
use tower_http::{trace::TraceLayer, timeout::TimeoutLayer};
use tower::ServiceBuilder;

struct Service {
    mysql_pool :mysql::MySqlPool,
    redis_pool : redis::Client,
}

impl Service {
    async fn new() -> Self {
        Service {
            mysql_pool: mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .connect("mysql://mysql:123456@localhost/test").await.unwrap(),
            redis_pool: redis::Client::open("redis://127.0.0.1/").unwrap(),
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
                .layer(TimeoutLayer::new(Duration::new(5, 0)))
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
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&state.mysql_pool)
        .await
        .unwrap();

    let mut con = state.redis_pool.get_async_connection().await.unwrap();

    let _: () = con.set("key1", b"foo").await.unwrap();

    let _: () = redis::cmd("SET")
        .arg(&["key2", "bar"])
        .query_async(&mut con)
        .await.unwrap();

    let result = redis::cmd("MGET")
        .arg(&["key1", "key2"])
        .query_async(&mut con)
        .await;
    assert_eq!(result, Ok(("foo".to_string(), b"bar".to_vec())));
    dbg!(&row);
    Json(json!({ "data": 42 }))
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