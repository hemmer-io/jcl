# Small configuration file (~10 lines)
# Used for baseline performance testing

app_name = "MyApp"
version = "1.0.0"
environment = "production"

database = (
    host = "localhost",
    port = 5432,
    name = "myapp_db"
)

features = ["auth", "api", "websocket"]
