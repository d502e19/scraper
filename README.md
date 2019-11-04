# Scraper ![TravisCI](https://travis-ci.org/d502e19/scraper.svg?branch=master) ![GitHub release](https://img.shields.io/github/release/d502e19/scraper.svg) ![GitHub](https://img.shields.io/github/license/d502e19/scraper.svg)

A distributed web scraper

## Usage

### Worker module
The worker module takes, in prioritised order; CLI arguments, environment variables, and lastly default values. See the following help-message:
```
DatScraper 0.1.0
d502e19@aau

USAGE:
    worker [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --addr <ADDR>                Specify the manager's address [env: SCRAPER_MANAGER_ADDRESS=]  [default: localhost]
    -e, --rmq-exchange <EXCHANGE>    Specify the RabbitMQ exchange to connect to [env: SCRAPER_RABBITMQ_EXCHANGE=]
                                     [default: work]
    -p, --rmq-port <PORT>            Specify the RabbitMQ port to connect to [env: SCRAPER_RABBITMQ_PORT=]  [default:
                                     5672]
    -q, --rmq-queue <QUEUE>          Specify the RabbitMQ queue to connect to [env: SCRAPER_RABBITMQ_QUEUE=]  [default:
                                     frontier]
    -k, --rmq-routing-key <KEY>      Specify the RabbitMQ routing-key to connect to [env: SCRAPER_RABBITMQ_ROUTING_KEY=]
                                     [default: ]
    -r, --redis-port <PORT>          Specify the redis-port to connect to [env: SCRAPER_REDIS_PORT=]  [default: 6379]
    -s, --redis-set <SET>            Specify the redis set to connect to [env: SCRAPER_REDIS_SET=]  [default:
                                     collection]
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
    -d, --rmq-redis-queue <QUEUE>    Specify the RabbitMQ-REDIS queue to connect to [env: SCRAPER_RABBITMQ_REDIS_QUEUE=]
                                     [default: collection]
    -t, --rmq-consumer-tag <TAG>     Specify the RabbitMQ consumer tag to use [env: SCRAPER_RABBITMQ_CONSUMER_TAG=]
                                     [default: proxy]
    -a, --addr <ADDR>                Specify the redis address [env: SCRAPER_REDIS_ADDRESS=]  [default: localhost]
    -r, --redis-port <PORT>          Specify the redis-port to connect to [env: SCRAPER_REDIS_PORT=]  [default: 6379]
    -s, --redis-set <SET>            Specify the redis set to connect to [env: SCRAPER_REDIS_SET=]  [default:
                                     collection]
```
