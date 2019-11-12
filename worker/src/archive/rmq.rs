use futures::Future;
use lapin_futures::{BasicProperties, Channel};
use lapin_futures::options::BasicPublishOptions;

use crate::errors::{ArchiveError, ArchiveResult};
use crate::traits::Archive;
use crate::errors::ArchiveErrorKind::UnreachableError;

/// The RabbitMQArchive is an Archive that stores the found data in a RabbitMQ queue
#[allow(dead_code)]
pub struct RabbitMQArchive {
    channel: Channel,
    exchange: String,
    routing_key: String,
}

#[allow(dead_code)]
impl RabbitMQArchive {
    /// Construct a new RabbitMQArchive
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
    fn archive_content(&self, content: D) -> ArchiveResult<()> {
        // Submit data to RabbitMQ
        let bytes = content.into();
        let res = self.channel.basic_publish(
            self.exchange.as_str(),
            self.routing_key.as_str(),
            bytes,
            BasicPublishOptions::default(),
            BasicProperties::default(),
        ).wait();

        res.map_err(|e| ArchiveError::new(UnreachableError, "Could not archive to RabbitMQ", Some(Box::new(e))))
    }
}