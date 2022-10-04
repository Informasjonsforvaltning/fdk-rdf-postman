use std::time::Duration;

use fdk_rdf_postman::{
    error::Error,
    kafka::{handle_message, BROKERS},
};
use rdkafka::{
    consumer::{CommitMode, Consumer, StreamConsumer},
    error::KafkaError,
    message::BorrowedMessage,
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use schema_registry_converter::{
    async_impl::{
        avro::{AvroDecoder, AvroEncoder},
        schema_registry::SrSettings,
    },
    schema_registry_common::SubjectNameStrategy,
};
use serde::Serialize;

/// Consumes all messages and drops their content.
pub async fn consume_all_messages(consumer: &StreamConsumer) -> Result<(), KafkaError> {
    loop {
        // Loop untill no nessage can be received within timeout.
        let timeout_duration = Duration::from_millis(500);
        if let None = consume_single_message(consumer, timeout_duration).await? {
            return Ok(());
        }
    }
}

/// Consumes and returns a single message, if received within the timeout period.
pub async fn consume_single_message(
    consumer: &StreamConsumer,
    timeout_duration: Duration,
) -> Result<Option<BorrowedMessage>, KafkaError> {
    match tokio::time::timeout(timeout_duration, consumer.recv()).await {
        Ok(result) => {
            let message = result?;
            // Commit offset back to kafka.
            consumer.commit_message(&message, CommitMode::Sync)?;
            Ok(Some(message))
        }
        // Timeout while trying to receive new message.
        Err(_) => Ok(None),
    }
}

pub async fn process_single_message(consumer: StreamConsumer) -> Result<(), Error> {
    let mut decoder = AvroDecoder::new(sr_settings());
    let http_client = reqwest::Client::new();

    let timeout_duration = Duration::from_millis(3000);
    let message = consume_single_message(&consumer, timeout_duration)
        .await?
        .expect("no message received within timeout duration");

    handle_message(
        &mut decoder,
        &message,
        &http_client,
    )
        .await
}

pub fn sr_settings() -> SrSettings {
    let schema_registry = "http://localhost:8081";
    SrSettings::new_builder(schema_registry.to_string())
        .set_timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}

pub struct TestProducer<'a> {
    producer: FutureProducer,
    encoder: AvroEncoder<'a>,
    topic: &'static str,
}

impl TestProducer<'_> {
    pub fn new(topic: &'static str) -> Self {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", BROKERS.clone())
            .create::<FutureProducer>()
            .expect("Failed to create Kafka FutureProducer");

        let encoder = AvroEncoder::new(sr_settings());
        Self {
            producer,
            encoder,
            topic,
        }
    }

    pub async fn produce<I: Serialize>(&mut self, item: I, schema: &str) {
        let encoded = self
            .encoder
            .encode_struct(
                item,
                &SubjectNameStrategy::RecordNameStrategy(schema.to_string()),
            )
            .await
            .unwrap();
        let record: FutureRecord<String, Vec<u8>> = FutureRecord::to(self.topic).payload(&encoded);
        self.producer
            .send(record, Duration::from_secs(0))
            .await
            .unwrap();
    }
}
