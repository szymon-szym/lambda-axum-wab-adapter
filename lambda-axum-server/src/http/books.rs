use std::collections::HashMap;

use axum::{extract::{Path, State}, Json};
use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};
use serde_dynamo::from_item;
use uuid::Uuid;

// models

#[derive(Serialize, Deserialize)]
pub struct Book {
    id: Uuid,
    author: String,
    title: String,
    #[serde(rename="yearOfPublishing")]
    year: DateTime<Utc>,
    description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BookInput {
    author: String,
    title: String,
    #[serde(rename="yearOfPublishing")]
    year: DateTime<Utc>,
    description: Option<String>,
}

// handlers

async fn get_book(ctx: State<AppContext>, Path(id): Path<Uuid>) -> Json<Book> {
    
    let result = ctx.dynamodb_client.get_item()
        .table_name(&ctx.table_name)
        .key("id", aws_sdk_dynamodb::types::AttributeValue::S(id.to_string()))
        .send().await.unwrap();

    let item = result.item().unwrap();

    let book: Book = from_item(item.clone()).unwrap();

    Json(book)
}

async fn create_book(ctx: State<AppContext>, Json(input): Json<BookInput>) -> Json<Book> {
    let book = Book {
        id: Uuid::new_v4(),
        author: input.author,
        title: input.title,
        year: input.year,
        description: input.description,
    };

    let item = serde_dynamo::to_item(&book).unwrap();

    let _ = ctx.dynamodb_client.put_item()
        .set_item(Some(item))
        .table_name(ctx.table_name.clone())
        .send()
        .await.unwrap();

    Json(book)
}

async fn update_book(ctx: State<AppContext>, Path(id): Path<Uuid>, Json(input): Json<BookInput>) -> Json<Book> {
    let book = Book {
        id,
        author: input.author,
        title: input.title,
        year: input.year,
        description: input.description,
    };

    let update_expression = "set author = :author, title = :title, yearOfPublishing = :yearOfPublishing, description = :description";

    let expression_attribute_values = HashMap::from([
        (String::from(":author"), aws_sdk_dynamodb::types::AttributeValue::S(book.author.clone())),
        (String::from(":title"), aws_sdk_dynamodb::types::AttributeValue::S(book.title.clone())),
        (String::from(":yearOfPublishing"), aws_sdk_dynamodb::types::AttributeValue::S(book.year.to_string())),
        (String::from(":description"), aws_sdk_dynamodb::types::AttributeValue::S(book.description.clone().unwrap_or_default())),
    ]);
    
    let _ = ctx.dynamodb_client.update_item()
        .table_name(&ctx.table_name)
        .key("id", aws_sdk_dynamodb::types::AttributeValue::S(id.to_string()))
        .update_expression(update_expression)
        .set_expression_attribute_values(Some(expression_attribute_values))
        .send()
        .await.unwrap();

    Json(book)
}

async fn delete_book(ctx: State<AppContext>, Path(_id): Path<Uuid>) -> () {

    let _ = ctx.dynamodb_client.delete_item()
        .table_name(&ctx.table_name)
        .key("id", aws_sdk_dynamodb::types::AttributeValue::S(_id.to_string()))
        .send().await.unwrap();
    ()
}

// router

#[derive(Clone)]
pub struct AppContext {
    table_name: String,
    dynamodb_client: aws_sdk_dynamodb::Client,
}


pub(crate) fn router(table_name: String, dynamodb_client: aws_sdk_dynamodb::Client) -> axum::Router {

    let app_state = AppContext {
        table_name,
        dynamodb_client
    };
    
    axum::Router::new()
        .route("/books/:id", axum::routing::get(get_book))
        .route("/books", axum::routing::post(create_book))
        .route("/books/:id", axum::routing::put(update_book))
        .route("/books/:id", axum::routing::delete(delete_book))
        .with_state(app_state)
}

