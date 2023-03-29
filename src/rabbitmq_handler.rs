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

        // Start a consumer.
        let consumer = queue.consume(ConsumerOptions::default()).unwrap();

        let message = consumer.receiver().recv().unwrap();
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);
                // println!("Received [{}]", body);
                let s = body.to_string();
                consumer.ack(delivery).unwrap();
                Some(s)
            }
            _ => None,
        }
    }
}
