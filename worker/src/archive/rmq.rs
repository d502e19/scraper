use lapin_futures::{Channel, BasicProperties};
use lapin_futures::options::BasicPublishOptions;
use crate::traits::Archive;
use std::error::Error;
use futures::Future;

pub struct RabbitMQArchive {
    channel: Channel,
    exchange: String,
    routing_key: String,
}

impl RabbitMQArchive {
    pub fn new(channel: Channel, exchange: String, routing_key: String) -> RabbitMQArchive {
        RabbitMQArchive {
            channel,
            exchange,
            routing_key,
        }
    }
}

impl<D> Archive<D> for RabbitMQArchive
    where D: Into<Vec<u8>> {
    fn archive_content(&self, content: D) -> Result<(), Box<dyn Error>> {
        let bytes = content.into();
        let res = self.channel.basic_publish(
            self.exchange.as_str(),
            self.routing_key.as_str(),
            bytes,
            BasicPublishOptions::default(),
            BasicProperties::default(),
        ).wait();

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}