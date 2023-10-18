use std::error::Error;

use lapin::{
    options::{BasicPublishOptions, ExchangeDeclareOptions},
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use log::trace;
use serde::Serialize;

use crate::data::models::Config;

pub struct Broker {
    connection: Connection,
    channel: Channel,
}

#[allow(dead_code)]
impl Broker {
    pub async fn connect(config: &Config) -> Result<Broker, Box<dyn Error>> {
        trace!(
            "Connecting -> {rabbitmq_uri}",
            rabbitmq_uri = config.rabbitmq_uri
        );

        let connection = Connection::connect(
            config.rabbitmq_uri.as_str(),
            ConnectionProperties::default(),
        )
        .await?;

        let channel = Broker::create_channel(&connection).await?;

        channel
            .exchange_declare(
                config.rabbitmq_exchange.as_str(),
                ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                Default::default(),
            )
            .await?;

        Ok(Broker {
            connection,
            channel,
        })
    }

    async fn create_channel(connection: &Connection) -> Result<Channel, Box<dyn Error>> {
        let channel = connection.create_channel().await?;

        Ok(channel)
    }

    pub async fn publish<T>(
        &self,
        exchange: &str,
        routing_key: &str,
        message: &T,
    ) -> Result<(), Box<dyn Error>>
    where
        T: ?Sized + Serialize,
    {
        trace!(
            "Publishing to exchange {exchange} with routing key {routing_key}",
            exchange = exchange,
            routing_key = routing_key,
        );

        let message = serde_json::to_string(&message).unwrap();

        self.channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                &message.into_bytes(),
                BasicProperties::default(),
            )
            .await?;

        Ok(())
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}
