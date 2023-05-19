## set environment variable

```shell
export PREPUBLISH_SQL_DB_URL="postgresql://your.postgresql/database_address"
export PREPUBLISH_MONGO_SRV_URL="mongodb://your.mongodb.server.address"
export PREPUBLISH_MONGO_DB_NM="your-mongo-database-name"
export PREPUBLISH_SRV_ADDR="127.0.0.1:8000"
```

## generate [entities](src/sql_entities) folder

```shell
cargo install sea-orm-cli
sea-orm-cli migrate refresh --database-url ${PREPUBLISH_SQL_DB_URL}
sea-orm-cli generate entity --with-serde both --entity-extra-derives schemars::JsonSchema --output-dir src/sql_entities --database-url ${PREPUBLISH_SQL_DB_URL}
```