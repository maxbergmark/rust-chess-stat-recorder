use std::time::Duration;
use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};

pub struct RabbitMqHandler;

impl RabbitMqHandler {
    pub fn get_filename_from_queue() -> Option<String> {
        let mut connection =
            Connection::insecure_open("amqp://guest:guest@192.168.10.200:30672").unwrap();

        // Open a channel - None says let the library choose the channel ID.
        let channel = connection.open_channel(None).unwrap();

        let options = QueueDeclareOptions {
            durable: true,
            exclusive: false,
            auto_delete: false,
            arguments: Default::default(),
        };

        let queue = channel.queue_declare("chess-files", options).unwrap();

        // Start a consumer
        let consumer = queue.consume(ConsumerOptions::default()).unwrap();

        let message = consumer.receiver()
            .recv_timeout(Duration::from_secs(1));

        match message {
            Ok(ConsumerMessage::Delivery(delivery)) => {
                let body = String::from_utf8_lossy(&delivery.body);
                let s = body.to_string();
                consumer.ack(delivery).unwrap();
                Some(s)
            }
            _ => None,
        }
    }
}
