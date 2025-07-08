# Response Handling

You can handle response using

```rust
use actix_web::{HttpResponse, Responder, web};
use actix_web::http::{Error, StatusCode};
use actix_web::http::header::ContentType;
```

Our modules implement response format which is a standard accross our system. Response can be

- Success Response
- Error Response

## System Overview

The key files in the response structure are

- **`src/utils/db_success_msgs.rs`**: Contains the messages of successful requests.
- **`src/utils/errors.rs`**: Contain errors which should be returned as per standard.
- **`src/utils/responses.rs`**: Contain response formats, for successful and error response.

## Usage

### Success Response

For generating a success response **`SuccessMessages`** from the *`db_success_msgs.rs`* is used. A related enum variant
must be set in it. The success message is generated and returned in **`services`** by the functions called by a *
*`handler`**. In the *`handler`* success response are generate using *`to_response`* funbction call for the
*`SuccessMessages`*. Another function *`create_success_response`* is returned by the handler in case of successful
response. The information returned by the *`to_response`* call is passed to the *`create_success_response`* function.

*`SuccessMessages`* are returned by services, which are then converted and returned by handlers.

#### Components mentioned

- Enum **`SuccessMessages`**
- Function implementation for the above, **`to_response`**
- Function to generate and return final success response, **`create_success_response`**

### Error Response

Generating erorr response follows the same flow as for the Success response.

#### Components of Error respose

- Enum **`Error`**
- Function implementation for the above, **`to_response`**
- Function to generate and return final success response, **`create_error_response`**

### Example Usage

Example of the response implementation can be found in the

- **`src/handlers/admins_api.rs`**
- **`src/services/admins_api.rs`**

> **Follow the same standard for returning responses. Services should return variants of either *`SuccessMessages`*
or *`Error` enums*. Handlers must use either *`create_success_response`* or *`create_error_response`***