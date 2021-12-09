use axum::{
    Json, Router,
    extract::{
        Form, Path, 
        ws::{
            Message, WebSocket, WebSocketUpgrade,
        },
        Extension,
    },
    http::StatusCode, response::{Html, IntoResponse},
    routing::{get, post, get_service},
    AddExtensionLayer,
};
use serde_derive::{Deserialize, Serialize};
use std::{collections::{HashMap,HashSet}, net::SocketAddr, str::FromStr,sync::{Arc, Mutex},};
//use tokio::runtime::Builder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tokio::sync::broadcast;

struct AppState {
    user_set: Mutex<HashSet<String>>,
    tx: broadcast::Sender<String>,
}


#[tokio::main]
async fn main() {
    //let rt = Builder::new_multi_thread().enable_all().build().unwrap();
    
    // ws
    let user_set = Mutex::new(HashSet::new());
    let (tx, _rx) = broadcast::channel(100);
    let app_state = Arc::new(AppState {user_set, tx});

    //rt.block_on(async {
        let app = Router::new()
        .nest(
            "/static",
            get_service(ServeDir::new("../static/")).handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            }),
        )
        .route("/", get(root))
        .route("/im", get(im))
        .route("/ws", get(websocket_handler))
        .route("/json", post(create_user_json))
        .route("/url/:name/:age", post(create_user_get))
        .route("/form", post(create_user_form))
        .layer(TraceLayer::new_for_http())
        .layer(AddExtensionLayer::new(app_state));

        let addr = SocketAddr::from_str("0.0.0.0:8882").unwrap();

        axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
    //});
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    // By splitting we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    // Username gets set in the receive loop, if it's valid.
    let mut username = String::new();

    // Loop until a text message is found.
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(name) = message {
            // If username that is sent by client is not taken, fill username string.
            check_username(&state, &mut username, &name);

            // If not empty we want to quit the loop else we want to quit function.
            if !username.is_empty() {
                break;
            } else {
                // Only send our client that username is taken.
                let _ = sender
                    .send(Message::Text(String::from("Username already taken.")))
                    .await;

                return;
            }
        }
    }

    // Subscribe before sending joined message.
    let mut rx = state.tx.subscribe();

    // Send joined message to all subscribers.
    let msg = format!("{} joined.", username);
    let _ = state.tx.send(msg);

    // This task will receive broadcast messages and send text message to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Clone things we want to pass to the receiving task.
    let tx = state.tx.clone();
    let name = username.clone();

    // This task will receive messages from client and send them to broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Add username before message.
            let _ = tx.send(format!("{}: {}", name, text));
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Send user left message.
    let msg = format!("{} left.", username);
    let _ = state.tx.send(msg);

    // Remove username from map so new clients can take it.
    state.user_set.lock().unwrap().remove(&username);
}

fn check_username(state: &AppState, string: &mut String, name: &str) {
    let mut user_set = state.user_set.lock().unwrap();

    if !user_set.contains(name) {
        user_set.insert(name.to_owned());

        string.push_str(name);
    }
}

async fn im() -> Html<&'static str> {
    Html(std::include_str!("../templates/index.html"))
}

async fn root() -> impl IntoResponse {
    "hello wolrd im !"
}

async fn create_user_get(Path(params): Path<HashMap<String, String>>) -> impl IntoResponse {
    let name = params.get("name").unwrap().clone();
    let age: i32 = params.get("age").unwrap().parse().unwrap();
    let user = User {
        id: 2000,
        name,
        age
    };

    (StatusCode::CREATED, Json(user))
}

async fn create_user_json (Json(payload): Json<CreateUser>) -> impl IntoResponse {
    let user = User {
        id: 1000,
        name: payload.name,
        age: payload.age
    };

    (StatusCode::CREATED, Json(user))
}

async fn create_user_form(Form(input): Form<CreateUser>) -> impl IntoResponse {
    let user = User{
        id: 3000,
        name: input.name,
        age: input.age,
    };
    
    (StatusCode::CREATED, Json(user))
}


#[derive(Deserialize)]
struct CreateUser {
    name: String,
    age: i32,
}

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    age: i32,
}