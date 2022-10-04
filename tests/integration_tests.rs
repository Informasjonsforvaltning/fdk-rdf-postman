use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr}
};

use fdk_rdf_postman::{
    kafka::{create_consumer, INPUT_TOPIC},
    schemas::{HarvestEvent, HarvestEventType},
};
use httptest::{
    matchers::{all_of, json_decoded, request, ExecutionContext, Matcher},
    responders::status_code,
    Expectation, Server, ServerBuilder,
};
use kafka_utils::{consume_all_messages, process_single_message, TestProducer};
use serde::{Deserialize, Serialize};

mod kafka_utils;

#[tokio::test]
async fn test() {
    let mut server = ServerBuilder::new()
        .bind_addr(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            8090,
        ))
        .run()
        .unwrap();

    assert_transformation(
        &server,
        "fdk-id",
        "\
        @prefix si: <https://www.w3schools.com/rdf/> .
        <https://digdir.no/dataset/007> si:author \"James Bond\" ;
            si:title \"The man!\" .
        ",
    ).await;

    // Assert that scoring api received expected requests.
    server.verify_and_clear();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffStoreGraph {
    pub id: String,
    pub graph: String,
}

impl Matcher<DiffStoreGraph> for DiffStoreGraph {
    fn matches(&mut self, expected: &DiffStoreGraph, _ctx: &mut ExecutionContext) -> bool {
        assert_eq!(expected.id, self.id);
        assert_eq!(expected.graph, self.graph);
        true
    }

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

async fn assert_transformation(
    server: &Server,
    id: &str,
    input: &str,
) {
    let consumer = create_consumer().unwrap();
    // Clear topic of all existing messages.
    consume_all_messages(&consumer).await.unwrap();
    // Start async node-namer process.
    let processor = process_single_message(consumer);

    // Create test event.
    let input_message = HarvestEvent {
        event_type: HarvestEventType::DatasetHarvested,
        timestamp: 1647698566000,
        fdk_id: id.to_string(),
        graph: input.to_string(),
    };

    let expected_body = DiffStoreGraph {
        id: id.to_string(),
        graph: input.to_string(),
    };

    server.expect(
        Expectation::matching(all_of![
            request::method("POST"),
            request::path("/api/graphs"),
            request::body(json_decoded::<DiffStoreGraph, DiffStoreGraph>(expected_body)),
        ])
            .respond_with(status_code(200)),
    );

    // Produce message to topic.
    TestProducer::new(&INPUT_TOPIC)
        .produce(&input_message, "no.fdk.dataset.DatasetEvent")
        .await;

    // Wait for node-namer to process message and assert result is ok.
    processor.await.unwrap();
}
