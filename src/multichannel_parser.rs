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

    pub(crate) fn start_consumer(&self) {
        let parsers: Vec<_> = (0..self.num_channels)
            .map(|i| ParallelParser::new(i, self.threads_per_channel))
            .collect();

        crossbeam::scope(|scope| {
            for parser in &parsers {
                scope.spawn(move |_| {
                    parser.create_channel();
                });
            }
        })
        .unwrap();
    }
}
