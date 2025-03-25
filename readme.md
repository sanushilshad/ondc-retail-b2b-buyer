
# ONDC RETAIL B2B BUYER APP
 
Backend server for an ONDC B2B Buyer App.
The progress can be tracked here: [milestones](#MILESTONES)


## Tech Stack
| Type | Technologies |
|---|---|
| Client | None |
| Server | Rust (Actix-web), Bash |
| Database | PostgreSQL |
| Caching | Redis, ElasticSearch |
| API Documention | OpenAPI Swagger |
| Messaging System | Apache Kafka |


## DEPENDENT MICROSERVIES

| Service Name | Description |
|---|---|
| Websocket Service | Service for all ONDC related Websocket connections |
| User Service | Service for all ONDC related User APIs |
| Short URL Service | Service for generating Short URL |
| Chat Service | Service for generating chat links |
| Payment Service | Service for payment |
| Utility Service | Service for common utility across all ONDC services |

## CUSTOM MIGRATION COMMAND FOR DEBUG:

```
cargo run --bin ondc-retail-b2b-buyer -- migrate
```

## CUSTOM MIGRATION COMMAND FOR RELEASE:

```
cargo run --release --bin  ondc-retail-b2b-buyer -- migrate
```

OR 

```
  ./target/release/ondc-retail-b2b-buyer migrate
```



## COMMAND FOR KAFKA TOPIC CREATION:

```
cargo run --release --bin  ondc-retail-b2b-buyer -- generate_kafka_topic
```
OR 

```
  ./target/release/ondc-retail-b2b-buyer generate_kafka_topic
```



### COMMAND FOR TOKEN GENERATION:
```
cargo run --bin ondc-retail-b2b-buyer -- generate_service_token
```
OR 

```
  ./target/release/ondc-retail-b2b-buyer generate_service_token
```

### COMMAND FOR ELASTIC SEARCH INDICES:
```

cargo run --bin ondc-retail-b2b-buyer -- generate_elastic_search_indices
```
OR 

```
  ./target/release/ondc-retail-b2b-buyer generate_elastic_search_indices
```



### COMMAND FOR GENERATING CACHE:
```
cargo run --bin ondc-retail-b2b-buyer -- generate_item_cache
```
OR 

```
  ./target/release/ondc-retail-b2b-buyer generate_item_cache
```

### COMMAND FOR REGENERATING CACHE FROM DATABASE:
```
cargo run --bin ondc-retail-b2b-buyer -- regenerate_item_cache
```
OR 

```
  ./target/release/ondc-retail-b2b-buyer regenerate_item_cache
```

## SQLX OFFLINE MODE:

```
cargo sqlx prepare
```

## ENVIRON VARIABLE 
- Set the following environ variables in files called `env.sh` and `configuration.yaml`.
- `env.sh`:
```

## DATABASE VARIABLES
export DATABASE__PASSWORD=""
export DATABASE__PORT=00
export DATABASE__HOST=""
export DATABASE__NAME=""
export DATABASE__USERNAME="postgres"
export DATABASE__DATABASE_URL="postgres://postgres:{password}@{host}:{port}/{db_name}"
export DATABASE__ACQUIRE_TIMEOUT=2
export DATABASE__MAX_CONNECTIONS=500
export DATABASE__MIN_CONNECTIONS=10

## EMAIL VARIABLES
export EMAIL__USERNAME=""
export EMAIL__PASSWORD=""
export EMAIL__BASE_URL=""
export EMAIL__SENDER_EMAIL=""
export EMAIL__TIMEOUT_MILLISECONDS=10000


## TARACING VARIABLES
export OTEL_SERVICE_NAME="ondc-retail-b2b-buyer"
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://localhost:4317"

## LOG VARIABLES
export TEST_LOG=True


## REDIS VARIABLE
export REDIS__HOST=""
export REDIS__PORT=""
export REDIS_PASSWORD=""


## ONDC GATEWAY VARIABLE
export ONDC__GATEWAY_URI="https://preprod.gateway.ondc.org"
export ONDC__REGISTRY_BASE_URL="https://preprod.registry.ondc.org/ondc"
export ONDC__OBSERVABILITY__TOKEN="123"
export ONDC__OBSERVABILITY__URL="3243"
export ONDC__OBSERVABILITY__IS_ENABLED=True
export ONDC__OBSERVABILITY__MAX_RETRIES=20
export ONDC__OBSERVABILITY__BACKOFF_VALUE=1

## APPLICATION DATA
export APPLICATION__NAME="ondc-retail-b2b-buyer"
export APPLICATION__ACCOUNT_NAME="dev2"
export APPLICATION__PORT=8228
export APPLICATION__HOST="0.0.0.0"
export APPLICATION__WORKERS=12
export APPLICATION__SERVICE_ID="40cf2e29-5964-4a4b-8228-51aa0081889a"

## SECRET VARIABLE
export SECRET__JWT__SECRET=""
export SECRET__JWT__EXPIRY=876600


## WEBSOCKET SERVICE
export WEBSOCKET__TOKEN=""
export WEBSOCKET__BASE_URL="http://0.0.0.0:8229"
export WEBSOCKET__TIMEOUT_MILLISECONDS=600000

## USER VARIABLE
export USER_OBJ__TOKEN=""
export USER_OBJ__BASE_URL="http://0.0.0.0:8230"
export USER_OBJ__TIMEOUT_MILLISECONDS=600000
export USER_OBJ__DEFAULT_USER_ID="ebce1f35-0fff-4b61-840d-fef8d43fd32b"
export USER_OBJ__DEFAULT_BUSINESS_ID="478b4366-f401-49fa-b6fc-8f9e23c15a1f"

## CHAT VARIABLE
export CHAT__TOKEN=""
export CHAT__BASE_URL="http://0.0.0.0:8232"
export CHAT__TIMEOUT_MILLISECONDS=600000

## KAFKA SERVICE
export KAFKA__SERVERS="kafka12:9091"
export KAFKA__ENVIRONMENT="test"

## ELASTICSEARCH SEARCH 
export ELASTIC_SEARCH__URL="https://0.0.0.0:9200"
export ELASTIC_SEARCH__ENV="test_preprod"
export ELASTIC_SEARCH__USERNAME="elastic"
export ELASTIC_SEARCH__PASSWORD="134"

## PAYMENT SERVICE
export PAYMENT__TOKEN=""
export PAYMENT__BASE_URL="http://0.0.0.0:5608"
export PAYMENT__TIMEOUT_MILLISECONDS=600000


```



- In order to verify SQL queries at compile time, set the below config in `.env` file:
```
export DATABASE_URL="postgres://postgres:{password}@{host}:{port}/{db_name}"

```

## TO RUN THE SERVER:

- For running development server:
```
bash dev_run.sh
```

- For running production server:
```
bash release.sh
```

- For restarting production server:
```
bash restart.sh
```

- For killing server:
```
bash kill.sh
```


## API DOCUMENTATION:
The API Docmentation can be found at `https://{{domain}}/docs/` after running the server.

## DEBUG SETUP:
- launch.json
```json
{

    "version": "0.2.0",
    "configurations": [

        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug executable 'ondc-retail-b2b-buyer'",
            "cargo": {
                "args": [
                    "build",
                    "--bin=ondc-retail-b2b-buyer",
                    "--package=ondc-retail-b2b-buyer"
                ],
                "filter": {
                    "name": "ondc-retail-b2b-buyer",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}",
            "envFile": "${workspaceFolder}/.env",
            "preLaunchTask": "cargo build",
        },
    ]
}
```
- settings.json

```json
{
    "[rust]": {
        "editor.formatOnSave": true,
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    },
    "editor.formatOnSave": true,
    "rust-analyzer.linkedProjects": [
        "./Cargo.toml"
    ],
}
```

- tasks.json
```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "cargo build",
            "type": "shell",
            "command": "cargo",
            "args": [
                "build",
                "--bin=ondc-retail-b2b-buyer",
                "--package=ondc-retail-b2b-buyer"
            ],
            "group": {
                "kind": "build",
                "isDefault": true
            },
            "problemMatcher": [
                "$rustc"
            ]
        }
    ]
}
```


## MILESTONES (54/56)
### MILESTONE 1 (Jan 18, 2023 - Jul 22, 2024):
* [x] Set up basic actix web server
* [x] Add environment config fetch
* [x] Develop custom migration command
* [x] Add tracing with Jaeger integration
* [x] Develop middleware to access request & response
* [x] Email service integration
* [x] Develop user registration API
* [x] Develop business registration API
* [x] Develop API for user authentication via password
* [x] Setting the codebase structure similar to Django
* [x] Develop JWT creation and verification middleware
* [x] Integrate Openapi swagger
* [x] Integrate Redis
* [x] Integrate Websocket 
* [x] Add business verification middleware
* [x] Add generic header validation middleware
* [x] Add seller auth header validation middleware
* [x] Develop realtime search API
* [x] Develop ONDC on search API
* [x] Fix integration test + add unit test cases for the apis in milestone 1
* [x] Add application release + debug + kill bash scripts + restart scripts

### MILESTONE 2: (Jul 23, 2024 - Nov 12, 2024)
* [x] Develop select API
* [x] Develop ondc on select API
* [x] Develop init API
* [x] Develop ONDC on init API
* [x] Develop confirm API
* [x] Develop ONDC on on_confirm API
* [x] Move Websocket as a seperate service
* [x] Move User module as a seperate service
* [x] Develop status API
* [x] Develop ONDC on status API
* [x] Develop cancel API
* [x] Develop ONDC on cancel API
* [x] Develop update API
* [x] Develop ONDC on update API

### MILESTONE 3: (Nov 14, 2024 - March 16 2025)
* [x] Update websocket implementation to enable Background Sync.
* [x] Remove device wise notification push for order flow.
* [x] Add Import flow to order.
* [x] Develop & Integrate permission flow.
* [x] Add Chat Functionality.
* [x] Integrate Address Module (Address Create and Fetch).
* [x] Develop Business Account fetch API
* [x] Develop User Account fetch API
* [x] Integrate Kafka to reduce load due to On-Search API.
* [x] Develop Business Specific Config Module.
* [x] Add Order series generation.
* [x] Integrate ElasticSearch.
* [x] Develop Order Meta Data Fetch API
* [x] Develop Order Detail Fetch API.
* [x] Integrate BAP Payment Gateway.
* [x] Develop Product Caching.
* [x] Develop Observability Module.

### MILESTONE 4:
* [x] Add basic validation for search/ select/ init/ confirm/ status/ cancel/ update APIs (Basic Validation).
* [x] Add basic validation ONDC for on_search/ on_select/ on_init/ on_confirm/ on_status/ on_cancel/ on_update APIs.
* [ ] Add test cases for milestone 2 (Only unit tests) (If I feel like it).
* [ ] Develop common Microservice to fetch (cities + country + states + category + category_attribute).


### MILESTONE 5 (OPTIONAL - when I have nothing better to do):
* [ ] Integrate with notification microservice to enable WhatsApp, Email and SMS functionality.
* [ ] Break the codebase to protocol and application micro services.

### THE MILESTONES ARE SUSCEPTIBLE TO CHANGES ╰(*°▽°*)╯