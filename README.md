# Trade Statistics

## Build

```bash
cargo build
```

## Run tests

```bash
cargo test
````

## Run the application

```bash
cargo run
```

## REST requests

### POST /add_batch

```bash
echo '{"symbol": "y", "values": [3, 3, 3]}' |  xh POST 'localhost:3000/add_batch'
```

### GET /stats

```bash
xh GET 'localhost:3000/stats?symbol=y&k=10'
```
