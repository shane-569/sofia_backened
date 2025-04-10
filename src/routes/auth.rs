use actix_web::{post, web, HttpResponse};
use mongodb::{bson::{doc, Document}, Database};
use serde::{Deserialize, Serialize};
use crate::{auth::jwt::generate_token, models::user::User};
use bcrypt::verify;
use crate::auth::jwt::validate_token;

#[derive(Deserialize)]
struct AuthRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
}

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[post("/signup")]
async fn signup(
    db: web::Data<Database>,
    payload: web::Json<AuthRequest>,
) -> HttpResponse {
    if payload.username.trim().is_empty() || payload.password.trim().is_empty() {
        return HttpResponse::BadRequest().body("Username and password cannot be empty");
    }

    let new_user = User::new(payload.username.clone(), payload.password.clone());
    let users = db.collection::<Document>("users");

    let doc = mongodb::bson::to_document(&new_user).unwrap();
    match users.insert_one(doc, None).await {
        Ok(_) => HttpResponse::Ok().body("User created successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[post("/login")]
async fn login(
    db: web::Data<Database>,
    payload: web::Json<AuthRequest>,
) -> HttpResponse {
    if payload.username.trim().is_empty() || payload.password.trim().is_empty() {
        return HttpResponse::BadRequest().body("Username and password cannot be empty");
    }

    let users = db.collection::<Document>("users");
    let filter = doc! { "username": &payload.username };

    match users.find_one(filter, None).await {
        Ok(Some(user_doc)) => {
            let db_hash = user_doc.get_str("password").unwrap_or("");
            if verify(&payload.password, db_hash).unwrap_or(false) {
                let id = user_doc.get_object_id("_id").unwrap().to_string();
                let access_token = generate_token(&id, 15);
                let refresh_token_value = generate_token(&id, 60);
                HttpResponse::Ok().json(TokenResponse { access_token,  refresh_token: refresh_token_value, })
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        _ => HttpResponse::Unauthorized().body("Invalid credentials"),
    }
}


#[post("/refresh")]
async fn refresh_token(
    payload: web::Json<RefreshRequest>,
) -> HttpResponse {
    match validate_token(&payload.refresh_token) {
        Ok(claims) => {
            let new_access_token = generate_token(&claims.sub, 15);
            HttpResponse::Ok().json(serde_json::json!({
                "access_token": new_access_token
            }))
        }
        Err(e) => {
            println!("[ERROR] Refresh token validation failed: {:?}", e);
            HttpResponse::Unauthorized().body("Invalid refresh token")
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").service(signup).service(login).service(refresh_token));
}