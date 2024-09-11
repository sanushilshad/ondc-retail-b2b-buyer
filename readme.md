
# ONDC RETAIL B2B BUYER APP
 
Backend server for an ONDC B2B Buyer App.
The progress can be tracked here: [milestones](#MILESTONES)


## Tech Stack
| Type | Technologies |
|---|---|
| Client | None |
| Server | Rust (Actix-web), Bash |
| Database | PostgreSQL |
| Caching | Redis, MeileiSeach |
| Email Service | Amazon Email Service |
| API Documention | OpenAPI Swagger |


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

## EMAIL VARIABLES
export EMAIL_CLIENT__USERNAME=""
export EMAIL_CLIENT__PASSWORD=""
export EMAIL_CLIENT__BASE_URL=""
export EMAIL_CLIENT__SENDER_EMAIL=""
export EMAIL_CLIENT__TIMEOUT_MILLISECONDS=10000


## TARACING VARIABLES
export OTEL_SERVICE_NAME="ondc-retail-b2b-buyer"
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://localhost:4317"
# export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318/v1/traces"
# export OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SERVER_REQUEST="X-Request-*"

## LOG VARIABLES
export TEST_LOG=True

## SECRET VALUES
export SECRET__JWT__SECRET=""

## REDIS VARIABLE
export REDIS__HOST=""
export REDIS__PORT=""
export REDIS_PASSWORD=""


## BUYER NP DETAILS
export ONDC__BAP__ID=""
export ONDC__BAP__URI=""
export ONDC__BAP__SIGNING_KEY=""

##USER VARIABLE
export LIST__USER__ADMIN_LIST="9562279968,"

## ONDC GATEWAY VARIABLE
export ONDC__GATEWAY_URI="https://preprod.gateway.ondc.org"
export ONDC__REGISTRY_BASE_URL="https://preprod.registry.ondc.org/ondc"

## PRODUCT MEILEI
export PRODUCT__MEILEI__API_MASTER_KEY=""
export PRODUCT__MEIELI__URL=""

## APPLICATION DATA
export APPLICATION__NAME="ondc-retail-b2b-buyer"


```

-  `configuration.yaml`:

```
application:
  port: 8002
  host: 0.0.0.0

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

## API DOCUMENTATION:
The API Docmentation can be found at `https://{{domain}}/docs/` after running the server.

## MILESTONES
### MILESTONE 1:
* [x] Set up basic actix web server
* [x] Add environment config fetch
* [x] Develop custom migration command
* [x] Add tracing with jaeger integration
* [x] Develop middleware to access request & response
* [x] Email service integration
* [x] Develop user registration API
* [x] Develop business registration API
* [x] Develop API for user authentication via password
* [x] Setting the codebase structure similar to django
* [x] Develop JWT creation and verification middleware
* [x] Integrate openapi swagger
* [x] Integrate Redis
* [x] Integrate Websocket 
* [x] Add business verification middleware
* [x] Add generic header validation middleware
* [x] Add realtime search API
* [x] Add seller auth header validation middleware
* [x] Add ONDC on search api
* [x] Fix integration test + add unit test cases for the apis in milestone 1
* [x] Add application release + debug + kill bash scripts 



### MILESTONE 2:
* [x] Develop select API
* [x] Develop ondc on select API
* [x] Develop init API
* [x] Develop ONDC on init API
* [ ] Develop confirm API
* [ ] Develop ONDC on on_confirm API
* [ ] Develop status API
* [ ] Develop ONDC on status API
* [ ] Develop cancel API
* [ ] Develop ONDC on cancel API
* [ ] Develop update API
* [ ] Develop ONDC on update API



### MILESTONE 3:
* [ ] Develop IGM issue API
* [ ] Develop ONDC on issue API
* [ ] Develop IGM issue_status API
* [ ] Develop ONDC on issue_status API
* [ ] Develop IGM issue close API


### MILESTONE 4:
* [ ] Develop info API
* [ ] Develop ONDC info API
* [ ] Integrate Payment Gateway
* [ ] Add Chat Functionality
* [ ] Develop Business Account fetch API
* [ ] Develop User Account fetch API
* [ ] Develop Business Account Update API
* [ ] Develop User Account Update API
* [ ] Develop Password Reset API
* [ ] Develop Order Fetch API
* [ ] Develop & Integrate permission flow
* [ ] Develop Permission assignment API
* [ ] Integrate MeileiSeach
* [ ] Integrate Redis PubSub With Webocket
* [ ] Integrate Pulser/Kafka/RabbitMQ
* [ ] Integrate with notification microservice to enable WhatsApp, Email and SMS functionality

### MILESTONE 4:
* [ ] Complete validation for business_account registration
* [ ] Complete validation for user_account registration
* [ ] Complete validation for product search registration
* [ ] Complete validation ONDC on search registration


### OPTIONAL

* [ ] Integrate SMS
* [ ] Integrate Whatsapp
* [ ] Develop email verfication APIs for user and business account
* [ ] Develop mobile verfication APIs for user and business account
* [ ] Develop sms otp API
* [ ] Develop email otp API
* [ ] Design product caching
* [ ] Develop new config fetch (will be given the last priority)
* [ ] Intergrate etcd for TSP flow (when i have nothing better to do: probably never)
### THE MILESTONES ARE SUSCEPTIBLE TO CHANGES ╰(*°▽°*)╯