# Scraper ![TravisCI](https://travis-ci.org/d502e19/scraper.svg?branch=master) ![GitHub release](https://img.shields.io/github/release/d502e19/scraper.svg) ![GitHub](https://img.shields.io/github/license/d502e19/scraper.svg)

A distributed web scraper

## Usage

### Worker module
The worker module takes, in prioritised order; CLI arguments, environment variables, and lastly default values. See the following help-message:
```
DatScraper Worker 0.1.0
d502e19@aau

USAGE:
    worker [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --filter-enable <BOOLEAN>          Specify whether filtering is enabled [env: SCRAPER_FILTER_ENABLE=]  [default:
                                           false]
    -w, --filter-path <PATH>               Specify path to list for filtering [env: SCRAPER_FILTER_PATH=]  [default:
                                           src/filter/whitelist.txt]
    -t, --filter-type <STRING>             Specify whether the list in the given filter-path is a 'white' or
                                           'black'-list [env: SCRAPER_FILTER_TYPE=]  [default: white]
    -o, --log-level <LEVEL>                Specify the log level {error, warn, info, debug, trace, off} [env:
                                           LOG_LEVEL=]  [default: info]
    -l, --log-path <PATH>                  Specify the log-file path [env: SCRAPER_WORKER_LOG_PATH=]  [default:
                                           worker.log]
    -d, --log-processing-time <BOOLEAN>    Specify whether to enable logging and calculating of finishing times for a
                                           task [env: SCRAPER_LOG_PROCESSING_TIME=true]  [default: false]
    -c, --rmq-collection <COLLECTION>      Specify the RabbitMQ collection queue to connect to [env:
                                           SCRAPER_RABBITMQ_COLLECTION_QUEUE=]  [default: collection]
    -e, --rmq-exchange <EXCHANGE>          Specify the RabbitMQ exchange to connect to [env: SCRAPER_RABBITMQ_EXCHANGE=]
                                           [default: work]
    -p, --rmq-port <PORT>                  Specify the RabbitMQ port to connect to [env: SCRAPER_RABBITMQ_PORT=]
                                           [default: 5672]
    -n, --rmq-prefetch-count <COUNT>       Specify the number of tasks to prefetch [env:
                                           SCRAPER_RABBITMQ_PREFETCH_COUNT=]  [default: 5]
    -q, --rmq-queue <QUEUE>                Specify the RabbitMQ queue to connect to [env: SCRAPER_RABBITMQ_QUEUE=]
                                           [default: frontier]
    -b, --redis-addr <ADDR>                Specify the Redis address [env: SCRAPER_REDIS_ADDRESS=]  [default: localhost]
    -r, --redis-port <PORT>                Specify the redis-port to connect to [env: SCRAPER_REDIS_PORT=]  [default:
                                           6379]
    -s, --redis-set <SET>                  Specify the redis set to connect to [env: SCRAPER_REDIS_SET=]  [default:
                                           collection]
    -a, --rmq-addr <ADDR>                  Specify the RabbitMQ address [env: SCRAPER_RMQ_ADDRESS=]  [default:
                                           localhost]
    -m, --sentinel <BOOLEAN>               Specify whether to use a sentinel redis connection or not [env:
                                           SCRAPER_SENTINEL=false]  [default: true]
```


### Redis Proxy module
The proxy module takes, in prioritised order; CLI arguments, environment variables, and lastly default values. See the following help-message:
```
DatScraper Proxy 0.1.0
d502e19@aau

USAGE:
    redis-proxy [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --log-level <LEVEL>          Specify the log level {error, warn, info, debug, trace, off} [env: LOG_LEVEL=]
                                     [default: info]
    -l, --log-path <PATH>            Specify the log-file path [env: SCRAPER_PROXY_LOG_PATH=]  [default: proxy.log]
    -d, --rmq-redis-queue <QUEUE>    Specify the RabbitMQ-REDIS queue to connect to [env: SCRAPER_RABBITMQ_REDIS_QUEUE=]
                                     [default: collection]
    -t, --rmq-consumer-tag <TAG>     Specify the RabbitMQ consumer tag to use [env: SCRAPER_RABBITMQ_CONSUMER_TAG=]
                                     [default: proxy]
    -k, --rmq-routing-key <KEY>      Specify the RabbitMQ routing-key to connect to [env: SCRAPER_RABBITMQ_ROUTING_KEY=]
                                     [default: ]
    -e, --addr <ADDR>                Specify the redis address [env: SCRAPER_REDIS_ADDRESS=]  [default: 192.168.99.100]
    -r, --redis-port <PORT>          Specify the redis-port to connect to [env: SCRAPER_REDIS_PORT=]  [default: 6379]
    -s, --redis-set <SET>            Specify the redis set to connect to [env: SCRAPER_REDIS_SET=]  [default:
                                     collection]
    -a, --rmq-addr <ADDR>            Specify the RabbitMQ address [env: SCRAPER_RMQ_ADDRESS=]  [default: 192.168.99.100]
    -p, --rmq-port <PORT>            Specify the RabbitMQ port to connect to [env: SCRAPER_RABBITMQ_PORT=]  [default:
                                     5672]
    -m, --sentinel <NAME>            An optional name of a master group for a sentinel Redis connection. [env:
                                     SCRAPER_SENTINEL=]  [default: none]
```
