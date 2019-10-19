use lapin_futures::{Channel, Queue};
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

impl <D>Archive<D> for RabbitMQArchive
    where D: Into<Vec<u8>>{
    fn archive_content(&self, content: D) -> Result<(), Box<dyn Error>> {
        let bytes = content.into();
        self.channel.basic_publish(
            self.exchange.as_str(),
            self.routing_key.as_str(),
            bytes,
            BasicPublishOptions::default(),
            BasicProperties::default(),
        ).wait()
    }
}