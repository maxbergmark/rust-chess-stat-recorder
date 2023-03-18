use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use crossbeam::channel::Sender;
use crossbeam::thread::Scope;
use cute::c;
use crate::parallel_parser::ParallelParser;

pub(crate) struct MultiChannelParser {
    num_channels: usize,
    threads_per_channel: usize,
}

impl MultiChannelParser {

    pub(crate) fn new(num_channels: i32, threads_per_channel: i32) -> Self {
        Self {
            num_channels: num_channels as usize,
            threads_per_channel: threads_per_channel as usize,
        }
    }

    fn spawn_queue_consumer_thread(&self, scope: &Scope, filename_send: Sender<String>) {

        scope.spawn(move |_| {
            // Open connection.
            let mut connection = Connection::insecure_open("amqp://guest:guest@192.168.10.200:30672").unwrap();

            // Open a channel - None says let the library choose the channel ID.
            let channel = connection.open_channel(None).unwrap();

            let declaration = QueueDeclareOptions {
                durable: true,
                exclusive: false,
                auto_delete: false,
                arguments: Default::default(),
            };

            // Declare the "hello" queue.
            let queue = channel.queue_declare("chess-files", declaration).unwrap();

            // Start a consumer.
            let consumer = queue.consume(ConsumerOptions::default()).unwrap();
            println!("Waiting for messages. Press Ctrl-C to exit.");

            for (i, message) in consumer.receiver().iter().enumerate() {
                match message {
                    ConsumerMessage::Delivery(delivery) => {
                        let body = String::from_utf8_lossy(&delivery.body);
                        println!("({:>3}) Received [{}]", i, body);
                        consumer.ack(delivery).unwrap();
                        filename_send.send(body.to_string()).unwrap();
                    }
                    other => {
                        println!("Consumer ended: {:?}", other);
                        break;
                    }
                }
            }

            // connection.close();
        });

    }

    pub(crate) fn start_consumer(&self) {

        let (filename_send, filename_recv) = crossbeam::channel::bounded(0);
        let parsers = c![ParallelParser::new(i, self.threads_per_channel), for i in 0..self.num_channels];
        // let parser = ParallelParser::new(0,8);
        crossbeam::scope(|scope| {
            self.spawn_queue_consumer_thread(scope, filename_send);
            for parser in &parsers {
                let filename_recv = filename_recv.clone();
                scope.spawn(move |_| {
                    parser.create_channel(filename_recv);
                });
                // parser.create_channel(scope, filename_recv.clone());
            }
        }).unwrap();
    }
}
/*
parser thread receives filename


 */