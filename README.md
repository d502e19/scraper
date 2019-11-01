# Scraper ![TravisCI](https://travis-ci.org/d502e19/scraper.svg?branch=master) ![GitHub release](https://img.shields.io/github/release/d502e19/scraper.svg) ![GitHub](https://img.shields.io/github/license/d502e19/scraper.svg)

A distributed web scraper

## Usage

### Worker module
The worker module takes, in prioritised order, CLI arguments, environment arguments, and lastly hardcoded default values. See the following help-message:
```
DatScraper 0.1.0
d502e19@aau

USAGE:
    worker [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --log-level <LEVEL>          Specify the log level {error, warn, info, debug, trace, off} [env: LOG_LEVEL=]
                                     [default: info]
    -l, --log-path <PATH>            Specify the log-file path [env: LOG_PATH=]  [default: worker.log]
    -e, --rmq-exchange <EXCHANGE>    Specify the RabbitMQ exchange to connect to [env: SCRAPER_RABBITMQ_EXCHANGE=]
                                     [default: work]
    -p, --rmq-port <PORT>            Specify the RabbitMQ port to connect to [env: SCRAPER_RABBITMQ_PORT=]  [default:
                                     5672]
    -q, --rmq-queue <QUEUE>          Specify the RabbitMQ queue to connect to [env: SCRAPER_RABBITMQ_QUEUE=]  [default:
                                     frontier]
    -k, --rmq-routing-key <KEY>      Specify the RabbitMQ routing-key to connect to [env: SCRAPER_RABBITMQ_ROUTING_KEY=]
                                     [default: ]
    -b, --redis-addr <ADDR>          Specify the Redis address [env: SCRAPER_REDIS_ADDRESS=]  [default: localhost]
    -r, --redis-port <PORT>          Specify the redis-port to connect to [env: SCRAPER_REDIS_PORT=]  [default: 6379]
    -s, --redis-set <SET>            Specify the redis set to connect to [env: SCRAPER_REDIS_SET=]  [default:
                                     collection]
    -a, --rmq-addr <ADDR>            Specify the RabbitMQ address [env: SCRAPER_RMQ_ADDRESS=]  [default: localhost]
```


### Redis Proxy module
