use actix_web::{get, web, HttpRequest, HttpResponse};
use mongodb::{bson::{doc, oid::ObjectId}, Database};
use mongodb::bson::Document;
use crate::auth::jwt::validate_token;
#[get("/me")]
async fn get_me(req: HttpRequest, db: web::Data<Database>) -> HttpResponse {
    let auth_header = req.headers().get("Authorization");

    if let Some(header_value) = auth_header {
        if let Ok(auth_str) = header_value.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                println!("[DEBUG] Token: {}", token);
                match validate_token(token) {
                    Ok(claims) => {
                        println!("[DEBUG] User ID from token: {}", claims.sub);
                        match ObjectId::parse_str(&claims.sub) {
                            Ok(obj_id) => {
                                println!("[DEBUG] Parsed ObjectId: {:?}", obj_id);
                                let users = db.collection::<Document>("users");
                                match users.find_one(doc! { "_id": obj_id }, None).await {
                                    Ok(Some(user)) => {
                                        let mut user = user.clone();
                                        user.remove("password");
                                        HttpResponse::Ok().json(user)
                                    },
                                    Ok(None) => {
                                        println!("[ERROR] User not found for ID: {}", claims.sub);
                                        HttpResponse::NotFound().body("User not found")
                                    },
                                    Err(e) => {
                                        println!("[ERROR] Database error: {}", e);
                                        HttpResponse::InternalServerError().finish()
                                    }
                                }
                            },
                            Err(e) => {
                                println!("[ERROR] Invalid ObjectId format: {} (Error: {})", claims.sub, e);
                                HttpResponse::BadRequest().body("Invalid user ID format")
                            }
                        }
                    },
                    Err(e) => {
                        println!("[ERROR] Token validation failed: {:?}", e);
                        HttpResponse::Unauthorized().body("Invalid token")
                    }
                }
            } else {
                HttpResponse::Unauthorized().body("Missing Bearer token")
            }
        } else {
            HttpResponse::Unauthorized().body("Invalid Authorization header")
        }
    } else {
        HttpResponse::Unauthorized().body("Missing Authorization header")
    }
}
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_me);
}
