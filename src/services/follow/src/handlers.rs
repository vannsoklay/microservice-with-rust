use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder};
use serde_json::json;

pub async fn follow(req: HttpRequest) -> impl Responder {
    let user_id = match req.extensions().get::<String>().cloned() {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json({
                json!({
                  "message": "Unauthorized missing user"
                })
            });
        }
    };
    HttpResponse::Ok().body("hello")
}

pub async fn unfollow(req: HttpRequest) -> impl Responder {
    let user_id = match req.extensions().get::<String>().cloned() {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json({
                json!({
                  "message": "Unauthorized missing user"
                })
            });
        }
    };
    HttpResponse::Ok().json(json!({
        "done":"done"
    }))
}

pub async fn follow_toggle(req: HttpRequest) -> impl Responder {
    let user_id = match req.extensions().get::<String>().cloned() {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json({
                json!({
                  "message": "Unauthorized missing user"
                })
            });
        }
    };
    HttpResponse::Ok().json(json!({
        "message":"working with follow and unfollow"
    }))
}

pub async fn follow_status(req: HttpRequest) -> impl Responder {
    let user_id = match req.extensions().get::<String>().cloned() {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json({
                json!({
                  "message": "Unauthorized missing user"
                })
            });
        }
    };
    HttpResponse::Ok().json(json!({
        "message": "get stutus follow"
    }))
}

pub async fn followers() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "message": "get folowers"
    }))
}

pub async fn following() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "message": "get following"
    }))
}

pub async fn count_follow() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "message": "get following"
    }))
}
