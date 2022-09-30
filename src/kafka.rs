use std::env;
use lazy_static::lazy_static;
use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
    message::BorrowedMessage,
    ClientConfig,
};
use crate::error::Error;

lazy_static! {
    pub static ref BROKERS: String = env::var("BROKERS").unwrap_or("localhost:9092".to_string());
    pub static ref INPUT_TOPIC: String =
        env::var("INPUT_TOPIC").unwrap_or("dataset-events".to_string());
}

pub fn create_consumer() -> Result<StreamConsumer, KafkaError> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "fdk_rdf_postman")
        .set("bootstrap.servers", BROKERS.clone())
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "false")
        .set("auto.offset.reset", "beginning")
        .set("api.version.request", "false")
        .set("debug", "all")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()?;
    consumer.subscribe(&[&INPUT_TOPIC])?;
    Ok(consumer)
}

pub async fn run_async_processor(worker_id: usize) -> Result<(), Error> {
    tracing::info!(worker_id, "starting worker");

    let consumer = create_consumer()?;

    tracing::info!(worker_id, "listening for messages");
    loop {
        let message = consumer.recv().await?;
        receive_message(&consumer, &message).await;
    }
}

async fn receive_message(
    consumer: &StreamConsumer,
    message: &BorrowedMessage<'_>,
) {
    let result = handle_message(message).await;
    match result {
        Ok(_) => {
            tracing::info!("message handled successfully");
        }
        Err(e) => {
            tracing::error!(
                error = e.to_string(),
                "failed while handling message"
            );
        }
    };
    if let Err(e) = consumer.store_offset_from_message(&message) {
        tracing::warn!(error = e.to_string(), "failed to store offset");
    };
}

async fn handle_message(message: &BorrowedMessage<'_>,) -> Result<(), Error> {
    tracing::info!("message! len: {}", message.payload_len());
    Ok(())
}
