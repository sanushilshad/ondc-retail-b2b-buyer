
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
| Email Service | Amazon Email Service |
| API Documention | OpenAPI Swagger |
| Messaging System | Apache Kafka |


## DEPENDENT MICROSERVIES

| Service Name | Description |
|---|---|
| Websocket Service | Service for all ONDC related Websocket connections |
| User Service | Service for all ONDC related User APIs |
| Short URL Service | Service for generating Short URL |
| Chat Service | Service for generating chat links |



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


## APPLICATION DATA
export APPLICATION__NAME="ondc-retail-b2b-buyer"
export APPLICATION__ACCOUNT_NAME="dev2"
export APPLICATION__PORT=8228
export APPLICATION__HOST="0.0.0.0"
export APPLICATION__WORKERS=12

## WEBSOCKET SERVICE
export WEBSOCKET__TOKEN=""
export WEBSOCKET__BASE_URL="http://0.0.0.0:8229"
export WEBSOCKET__TIMEOUT_MILLISECONDS=600000

##USER VARIABLE
export USER_OBJ__TOKEN=""
export USER_OBJ__BASE_URL="http://0.0.0.0:8230"
export USER_OBJ__TIMEOUT_MILLISECONDS=600000


## CHAT VARIABLE
export CHAT__TOKEN=""
export CHAT__BASE_URL="http://0.0.0.0:8232"
export CHAT__TIMEOUT_MILLISECONDS=600000

## KAFKA SERVICE
export KAFKA__SERVERS="kafka12:9091"
export KAFKA__SEARCH_TOPIC_NAME="test_ondc_search"


## ELASTICSEARCH SEARCH 
export ELASTIC_SEARCH__URL="https://0.0.0.0:9200"
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


## MILESTONES (47/68)
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

### MILESTONE 3: (Nov 14, 2024 - Pending)
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
* [ ] Develop Product Caching.
* [ ] Integrate BAP Payment Gateway.
* [ ] Develop and Integrate Observability Module.



### MILESTONE 4:
* [ ] Develop common Microservice to fetch city codes, etc.
* [ ] Add test cases for milestone 2 (Only unit tests)

### MILESTONE 5:
* [ ] Develop IGM issue API.
* [ ] Develop ONDC on issue API.
* [ ] Develop IGM issue_status API.
* [ ] Develop ONDC on issue_status API.
* [ ] Develop IGM issue close API.


### MILESTONE 6:
* [ ] Develop Order Meta Data Fetch API
* [ ] Develop Order Detail Fetch API
* [ ] Develop Cancellation Code Fetch API
* [ ] Integrate with notification microservice to enable WhatsApp, Email and SMS functionality

### MILESTONE 7:
* [ ] Complete validation for search/ select/ init/ confirm/ status/ cancel/ update APIs
* [ ] Complete validation ONDC for on_search/ on_select/ on_init/ on_confirm/ on_status/ on_cancel/ on_update APIs


### MILESTONE 8:
* [ ] Develop info API
* [ ] Develop ONDC info API
* [ ] Develop new config fetch (will be given the last priority)
* [ ] Intergrate etcd for TSP flow (when I have nothing better to do: probably never)
* [ ] Add limit to the number of failed authentication
### THE MILESTONES ARE SUSCEPTIBLE TO CHANGES ╰(*°▽°*)╯