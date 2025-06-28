use actix_web::web;
use crate::handlers::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            web::resource("/store")
                .route(web::post().to(create_store))
        );
        // .service(
        //     web::resource("/store/{id}")
        //         .route(web::get().to(get_store))
        //         .route(web::put().to(update_store))
        //         .route(web::delete().to(delete_store))
        // );
}