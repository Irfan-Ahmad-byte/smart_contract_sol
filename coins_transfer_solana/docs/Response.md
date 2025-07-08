# Response Handling

You can handle response using

```sh
use actix_web::{HttpResponse, Responder, web};
use actix_web::http::{Error, StatusCode};
use actix_web::http::header::ContentType;
```

Returning the rendered template

```sh
let mut context = tera::Context::new();
context.insert("name", "World");

let mut response = HttpResponse::Ok();
response.content_type(ContentType::html());
response.body(View::render(String::from("index"), context))
```

For returning a JSON response

```sh
let mut response = HttpResponse::Ok();
response.content_type(ContentType::json());
response.body(json!({"message": "Hello World"}))
```

For returning a text response

```sh
let mut response = HttpResponse::Ok();
response.content_type(ContentType::plaintext());
response.body("Hello World")
```

For returning a binary response

```sh
let mut response = HttpResponse::Ok();
response.content_type(ContentType::octet_stream());
response.body(vec![0, 1, 2, 3, 4, 5])
```

For returning a response with a custom status code

```sh
let mut response = HttpResponse::build(StatusCode::BAD_REQUEST);
response.content_type(ContentType::json());
response.body(json!({"message": "Hello World"}))
```

For returning a response with a custom status code and a custom message

```sh
let mut response = HttpResponse::build(StatusCode::BAD_REQUEST);
response.content_type(ContentType::json());
response.body(json!({"message": "Hello World"}))
```

For returning a response with a custom status code and a custom message

```sh
let mut response = HttpResponse::build(StatusCode::BAD_REQUEST);
response.content_type(ContentType::json());
response.body(json!({"message": "Hello World"}))
```
