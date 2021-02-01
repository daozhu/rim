use crate::Pool;
use crate::models::{ProductJson, PostProduct, Product};

use actix_web::{HttpResponse, Error, post, get, delete, web};
use anyhow::Result;
use diesel::{RunQueryDsl, delete};
use diesel::dsl::{insert_into};
use diesel::prelude::*;
use tera::{Tera, Context};

pub async fn home() -> Result<HttpResponse, Error> {
    let tera = Tera::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")
    ).unwrap();
    let ctx = Context::new();
    let back = tera.render("index.html", &ctx).unwrap();
    Ok(HttpResponse::Ok().body(back))
}

#[post("/add_product")]
pub async fn add_product(
    pool: web::Data<Pool>,
    item: web::Json<ProductJson>
) -> Result<HttpResponse, Error> {
    Ok(
        add_single_product(pool, item)
            .await
            .map(|product| HttpResponse::Created().json(product))
            .map_err(|_|HttpResponse::InternalServerError())?
    )
}

async fn add_single_product(
    pool: web::Data<Pool>,
    item: web::Json<ProductJson>
) -> Result<Product, diesel::result::Error> {
    use crate::schema::product::dsl::*;
    let db_connection = pool.get().unwrap();
    match product
        .filter(name.eq(&item.name))
        .first::<Product>(&db_connection) {
        Ok(result) => Ok(result),
        Err(_) => {
            let new_product = PostProduct{
                name: &item.name,
                title: &item.title,
                date_created: &format!("{}", chrono::Local::now().naive_local())
            };
            insert_into(product)
                .values(&new_product)
                .execute(&db_connection)
                .expect("Error saving new product");
            let result = product.order(id.desc())
                    .first(&db_connection).unwrap();
            Ok(result)
        }
    }
}

#[get("/get_all_product")]
pub async fn get_all_product(
    pool: web::Data<Pool>
) -> Result<HttpResponse, Error> {
    Ok(
        get_all(pool)
            .await
            .map(|product|HttpResponse::Ok().json(product))
            .map_err(|_|HttpResponse::InternalServerError())?
    )
}

async fn get_all(
    pool: web::Data<Pool>
) -> Result<Vec<Product>, diesel::result::Error> {
    use crate::schema::product::dsl::*;
    let db_connection = pool.get().unwrap();
    let result = product.load::<Product>(&db_connection)?;
    Ok(result)
}

#[delete("/del_product/{id}")]
pub async fn del_product(
    pool: web::Data<Pool>,
    path: web::Path<String>
) -> Result<HttpResponse, Error> {
    Ok(
        del(pool, path)
            .await
            .map(|product|HttpResponse::Ok().json(product))
            .map_err(|_|HttpResponse::InternalServerError())?
    )
}

async fn del(
    pool: web::Data<Pool>,
    path: web::Path<String>
) -> Result<usize, diesel::result::Error> {
    use crate::schema::product::dsl::*;
    let db_connection = pool.get().unwrap();
    let id_string = &path.0;
    let i:i32 = id_string.parse().unwrap();
    let result = delete(product.filter(id.eq(i))).execute(&db_connection)?;
    Ok(result)
}