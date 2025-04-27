Draft about actors, how they should look like code-wise, and the functionality:

Since this language is mostly about targeted towards the web developers, having this syntax looks amazing

```rust
actor WebServer {
    read,write connections = []

    on request(url: String) -> Response {
        match url {
            "/api" => json({ status: "ok" }),
            _ => html("<h1>Not Found</h1>")
        }
    }
}
```
