use std::{
    env,
    time::{Duration, Instant},
};
use apache_avro::schema::Name;
use lazy_static::lazy_static;
use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
    message::BorrowedMessage,
    ClientConfig, Message,
};
use schema_registry_converter::{
    async_impl::{
        avro::AvroDecoder,
        schema_registry::SrSettings,
    },
    avro_common::DecodeResult,
};
use crate::{
    error::Error,
    diff_store::update_diff_store,
    metrics::{PROCESSED_MESSAGES, PROCESSING_TIME},
    schemas::{HarvestEvent, InputEvent},
};

lazy_static! {
    pub static ref BROKERS: String = env::var("BROKERS").unwrap_or("localhost:9092".to_string());
    pub static ref SCHEMA_REGISTRY: String =
        env::var("SCHEMA_REGISTRY").unwrap_or("http://localhost:8081".to_string());
    pub static ref INPUT_TOPIC: String =
        env::var("INPUT_TOPIC").unwrap_or("dataset-events".to_string());
}

pub fn create_sr_settings() -> Result<SrSettings, Error> {
    let mut schema_registry_urls = SCHEMA_REGISTRY.split(',');

    let mut sr_settings_builder =
        SrSettings::new_builder(schema_registry_urls.next().unwrap_or_default().to_string());
    schema_registry_urls.for_each(|url| {
        sr_settings_builder.add_url(url.to_string());
    });

    let sr_settings = sr_settings_builder
        .set_timeout(Duration::from_secs(5))
        .build()?;
    Ok(sr_settings)
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

pub async fn run_async_processor(worker_id: usize, sr_settings: SrSettings) -> Result<(), Error> {
    tracing::info!(worker_id, "starting worker");

    let consumer = create_consumer()?;
    let mut decoder = AvroDecoder::new(sr_settings);
    let http_client = reqwest::Client::new();

    tracing::info!(worker_id, "listening for messages");
    loop {
        let message = consumer.recv().await?;
        receive_message(&consumer, &mut decoder, &message, &http_client).await;
    }
}

async fn receive_message(
    consumer: &StreamConsumer,
    decoder: &mut AvroDecoder<'_>,
    message: &BorrowedMessage<'_>,
    http_client: &reqwest::Client,
) {
    let start_time = Instant::now();
    let result = handle_message(decoder, message, http_client).await;
    let elapsed_seconds = start_time.elapsed().as_secs_f64();

    let metric_label = match result {
        Ok(_) => {
            tracing::info!(elapsed_seconds, "message handled successfully");
            PROCESSING_TIME.observe(elapsed_seconds);
            "success"
        }
        Err(e) => {
            tracing::error!(
                elapsed_seconds,
                error = e.to_string(),
                "failed while handling message"
            );
            "error"
        }
    };
    PROCESSED_MESSAGES.with_label_values(&[metric_label]).inc();

    if let Err(e) = consumer.store_offset_from_message(message) {
        tracing::warn!(error = e.to_string(), "failed to store offset");
    };
}

pub async fn handle_message(
    decoder: &mut AvroDecoder<'_>,
    message: &BorrowedMessage<'_>,
    http_client: &reqwest::Client,
) -> Result<(), Error> {
    match decode_message(decoder, message).await? {
        InputEvent::HarvestEvent(event) => {
            update_diff_store(event, http_client).await
        }
        InputEvent::Unknown { namespace, name } => {
            tracing::warn!(namespace, name, "skipping unknown event");
            Ok(())
        }
    }
}

async fn decode_message(
    decoder: &mut AvroDecoder<'_>,
    message: &BorrowedMessage<'_>,
) -> Result<InputEvent, Error> {
    match decoder.decode(message.payload()).await? {
        DecodeResult {
            name:
                Some(Name {
                     name,
                     namespace: Some(namespace),
                     ..
                 }),
            value,
        } => {
            let event = match (namespace.as_str(), name.as_str()) {
                ("no.fdk.concept", "ConceptEvent") => {
                    InputEvent::HarvestEvent(apache_avro::from_value::<HarvestEvent>(&value)?)
                }
                ("no.fdk.dataset", "DatasetEvent") => {
                    InputEvent::HarvestEvent(apache_avro::from_value::<HarvestEvent>(&value)?)
                }
                _ => InputEvent::Unknown { namespace, name },
            };
            Ok(event)
        }
        _ => Err("unable to identify event without namespace and name".into()),
    }
}
