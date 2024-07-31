## CUSTOM MIGRATION COMMAND:

```
cargo run --bin ondc-retail-b2b-buyer -- migrate
```
## SQLX OFFLINE MODE:

```
cargo sqlx prepare
```

## ENVIRON VARIABLE 
- Set the following environ variables in files called `env.sh` and `configuration.yaml`.
- The value in `env.sh` are:
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

- The value in `configuration.yaml` are:

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


## MILESTONE 1:
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
* [x] Integrate redis
* [x] Integrate websocket 
* [x] Add business verification middleware
* [x] Add tsp changes into user account and business account
* [x] Add generic header validation middleware
* [x] Add realtime search API
* [x] Add seller auth header validation middleware
* [x] Add ondc on search api
* [x] Fix integration test + add unit test cases for the apis in milestone 1
* [x] Add application release + debug + kill bash scripts 



## MILESTONE 2:
* [ ] Develop select API
* [ ] Develop ondc on select API
* [ ] Develop init API
* [ ] Develop ondc on init API
* [ ] Develop confirm API
* [ ] Develop ondc on on_confirm API
* [ ] Develop status API
* [ ] Develop ondc on status API
* [ ] Develop update API
* [ ] Develop ondc on update API
* [ ] Develop IGM issue API
* [ ] Develop ondc on issue API
* [ ] Develop IGM issue_status API
* [ ] Develop ondc on issue_status API
* [ ] Develop IGM issue close API
* [ ] Integrate permission flow

## MILESTONE 3:
* [ ] Integrate permission flow
* [ ] Develop permission flow

## MILESTONE 4:
* [ ] Complete validation for business_account registration
* [ ] Complete validation for user_account registration
* [ ] Complete validation for product search registration
* [ ] Complete validation ondc on search registration

## OPTIONAL

* [ ] Integrate SMS
* [ ] Integrate Whatsapp
* [ ] Develop email verfication apis for user and business account
* [ ] Develop mobile verfication apis for user and business account
* [ ] Develop sms otp api
* [ ] Develop email otp api
* [ ] Design product caching
* [ ] Develop new config fetch (will be given the last priority)
* [ ] Intergrate etcd for tsp flow (when i have nothing better to do: probably never)
### THE MILESTONES ARE SUSCEPTIBLE TO CHANGES ╰(*°▽°*)╯