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
