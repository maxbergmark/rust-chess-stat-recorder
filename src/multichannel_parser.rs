use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use crossbeam::channel::Sender;
use crossbeam::thread::Scope;
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

    fn receive_queue_message() -> Option<String> {
        // Open connection.
        let mut connection = Connection::insecure_open("amqp://guest:guest@192.168.10.200:30672").unwrap();

        // Open a channel - None says let the library choose the channel ID.
        let channel = connection.open_channel(None).unwrap();

        let options = QueueDeclareOptions {
            durable: true,
            exclusive: false,
            auto_delete: false,
            arguments: Default::default(),
        };

        // Declare the "hello" queue.
        let queue = channel.queue_declare("chess-files", options).unwrap();

        // Start a consumer.
        let consumer = queue.consume(ConsumerOptions::default()).unwrap();

        let message = consumer.receiver().recv().unwrap();
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);
                println!("Received [{}]", body);
                let s = body.to_string();
                consumer.ack(delivery).unwrap();
                Some(s)
            }
            _ => {None}
        }
    }

    fn spawn_queue_consumer_thread(&self, scope: &Scope, filename_send: Sender<String>) {

        scope.spawn(move |_| {
            loop {
                let message = Self::receive_queue_message();
                match message {
                    Some(s) => {
                        filename_send.send(s).unwrap();
                    }
                    None => {
                        println!("Consumer ended");
                        break;
                    }
                }
            }
        });

    }

    pub(crate) fn start_consumer(&self) {

        let (filename_send, filename_recv) = crossbeam::channel::bounded(0);
        let parsers: Vec<_> = (0..self.num_channels)
            .map(|i| ParallelParser::new(i, self.threads_per_channel))
            .collect();
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