# RSS Feeder
🚧 In development, not for production use

## Purpose
Subscribe to RSS feeds and load into RabbitMQ

## Environment variables
Initial values.
| Req | Name | Description | Default |
| ---- | -------- | ----------- | ------- |
| | `RUST_LOG` | Logging level | `error` |
| | `INIT_RSS_PROXY` |Full text RSS | `http://ftr.fivefilters.org/makefulltextfeed.php?url=` | 
|✅| `INIT_RABBITMQ_URI` | RabbitMQ connection string | `amqp://guest:guest@localhost:5672/%2f` |
| | `INIT_RSS_FEEDS` | RSS Url,Name,Cron (semicolon separator for few feeds) | `http://feeds.bbci.co.uk/news/world/rss.xml,BBC News,0 */5 * * * *` |
| | `INIT_RABBITMQ_EXCHANGE` | Exchange to send feed to | `rss` |
| | `INIT_RABBITMQ_ROUTING_KEY` | Routing key for exchange | `inbox` |

`.env` file can be used if running from VSCode (launch.json)

For more about logger configuration follow [env_logger crate documentation](https://docs.rs/env_logger/0.10.0/env_logger/#enabling-logging)

## Used packages
Look into Cargo.toml
