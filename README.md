

# 1- HTTP Echo Server

This project demonstrates an HTTP server built using the low-level [Hyper](https://hyper.rs/) library. The server supports basic request routing and processes incoming HTTP requests.  

## Features  

- **Routing Table:**  
  - `GET /` → Returns a welcome message with available APIs.  
  - `POST /echo` → Returns the same body received in the request.  
  - `POST /echo/uppercase` → Converts the request body to uppercase before responding.  
  - `POST /echo/reverse` → Reverses the request body before responding.  
- **Handles HTTP Methods:** Supports `GET` and `POST`.  
- **Payload Size Protection:** Prevents excessive payloads by limiting the request body size to `64KB`.  
- **Asynchronous Processing:** Uses Tokio and Hyper for efficient async request handling.  

## Running the Server  

`cd network/`
```sh
cargo run --bin echo-server
```

By default, the server listens on `127.0.0.1:5948`.  

## HTTP API Testing

Postman & cURL request files are available in the raw/ directory. 

#### 1. Check available APIs  
```sh
curl -X GET http://127.0.0.1:5948/
```
**Response:**  
```
Welcome, We have the following apis:
POST echo, POST echo/uppercase, POST echo/reverse
```

#### 2. Echo Back the Request Body  
```sh
curl -X POST http://127.0.0.1:5948/echo -d "Hello, world!"
```
**Response:**  
```
Hello, world!
```

#### 3. Convert Request Body to Uppercase  
```sh
curl -X POST http://127.0.0.1:5948/echo/uppercase -d "hello"
```
**Response:**  
```
HELLO
```

#### 4. Reverse the Request Body  
```sh
curl -X POST http://127.0.0.1:5948/echo/reverse -d "Hello"
```
**Response:**  
```
olleH
```

#### 5. Invalid Route Example  
```sh
curl -X POST http://127.0.0.1:5948/unknown
```
**Response:**  
```
404 Not Found
```
