mod middewares;

use crate::middewares::{User, get_jwt};
use middewares::decode_jwt;
use serde_json::json;
use tokio::net::TcpListener;

use axum::{async_trait, extract::FromRequestParts, http::{header, request::Parts, StatusCode}, response::Response, routing::{get, post}, Json, Router};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let router = Router::new()
        .route("/public-view", get(public_view_handler))
        .route("/get-token", post(get_token_handler))
        .route("/secret-view", get(secret_view_handler));

    let tcp_listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Address should be free and valid");

    axum::serve(tcp_listener, router)
        .await
        .expect("Error serving application")
}

async fn public_view_handler() -> Response<String> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(
            json!({
                "success": true,
                "data": {
                    "message": "This data is available to all users"
                }
            })
            .to_string(),
        )
        .unwrap_or_default()
}

async fn get_token_handler(Json(user): Json<User>) -> Response<String> {
    let token = get_jwt(user);

    match token {
        Ok(token) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(
                json!({
                    "success": true,
                    "data": {
                        "token": token
                    }
                })
                .to_string(),
            )
            .unwrap_or_default(),

        Err(e) => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(
                json!({
                    "success": false,
                    "data": {
                        "message": e
                    }
                })
                .to_string(),
            )
            .unwrap_or_default(),
    }
}

async fn secret_view_handler(Auth(user): Auth) -> Response<String> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(
            json!({
                "success": true,
                "data": user
            })
            .to_string(),
        )
        .unwrap_or_default()
}

struct Auth(User);

#[async_trait]
impl<S> FromRequestParts<S> for Auth where S: Send + Sync {
    type Rejection = Response<String>;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let access_token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|str| str.split(" ").nth(1));

        match access_token {
            Some(token) => {
                let user = decode_jwt(token);

                match user {
                    Ok(user) => Ok(Auth(user)),
                    Err(e) => Err(
                        Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(
                            json!({
                                "success": false,
                                "data": {
                                    "message": e
                                }
                            })
                            .to_string(),
                        )
                        .unwrap_or_default(),
                    )
                }
            },
            None => Err(
                Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(
                        json!({
                            "success": false,
                            "data": {
                                "message": "No token provided"
                            }
                        })
                        .to_string(),
                    )
                    .unwrap_or_default(),
            ),
        }
    }
}