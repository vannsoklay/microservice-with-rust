use crate::{db::DBConfig, identify::identify, models::Product};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use futures_util::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId};

// get all products
pub async fn get_all_products() -> impl Responder {
    let collection = DBConfig::product_collection().await;
    let cursor = collection.find(doc! {}).await;

    match cursor {
        Ok(mut cursor) => {
            let mut products = vec![];
            while let Some(product) = cursor.try_next().await.unwrap_or(None) {
                products.push(product);
            }
            println!("products {:?}", products);
            HttpResponse::Ok().json(products)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// get product by id
pub async fn get_product_by_id(product_id: web::Path<String>) -> impl Responder {
    let collection = DBConfig::product_collection().await;
    let id = ObjectId::parse_str(&*product_id).unwrap();

    let product = collection.find_one(doc! { "_id": id }).await;

    match product.unwrap() {
        Some(product) => HttpResponse::Ok().json(product),
        None => HttpResponse::NotFound().finish(),
    }
}

// create a new product
pub async fn create_product(req: HttpRequest, new_product: web::Json<Product>) -> impl Responder {
    let collection = DBConfig::product_collection().await;

    let user_id = match identify(req).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    println!("user_id {}", user_id);

    let new_product = Product {
        id: None, // Let MongoDB generate the ID
        name: new_product.name.clone(),
        description: new_product.description.clone(),
        price: new_product.price,
        stock: new_product.stock,
    };

    let result = collection.insert_one(new_product).await;

    match result {
        Ok(insert_result) => HttpResponse::Created().json(insert_result.inserted_id),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// update a product
pub async fn update_product(
    product_id: web::Path<String>,
    updated_product: web::Json<Product>,
) -> impl Responder {
    let collection = DBConfig::product_collection().await;
    let id = ObjectId::parse_str(&*product_id).unwrap();

    let result = collection
        .update_one(
            doc! { "_id": id },
            doc! {
                "$set": {
                    "name": updated_product.name.clone(),
                    "description": updated_product.description.clone(),
                    "price": updated_product.price,
                    "stock": updated_product.stock
                }
            },
        )
        .await;

    match result {
        Ok(update_result) => {
            if update_result.matched_count == 1 {
                HttpResponse::Ok().json(update_result)
            } else {
                HttpResponse::NotFound().finish()
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// delete a product
pub async fn delete_product(product_id: web::Path<String>) -> impl Responder {
    let collection = DBConfig::product_collection().await;
    let id = ObjectId::parse_str(&*product_id).unwrap();

    let result = collection.delete_one(doc! { "_id": id }).await;

    match result {
        Ok(delete_result) => {
            if delete_result.deleted_count == 1 {
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::NotFound().finish()
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
