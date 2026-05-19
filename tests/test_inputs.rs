use std::{collections::HashMap, rc::Rc};

use anonit::{
    config::Config,
    data::{UpdateStructureData, value::DataValue},
    fake::{FakeDataProducer, MockFakeDataProducer, ignore::IgnoreFakeData},
    process_json_file,
};

use crate::common::key::path_key;

mod common;

/// Tests that when internal mappings are enabled that all internal
/// references are maintained
#[test]
fn test_internal_mappings_enabled() {
    let sample = include_str!("./samples/input_internal_mappings.json");
    let expected_output = include_str!("./samples/output_internal_mappings.json");

    // Fixed set of IDs to use when producing the mapping
    let ids: [&'static str; _] = [
        "test-post",
        "test-user-1",
        "test-comment-1",
        "test-comment-2",
        "test-user-2",
    ];
    let mut id_iter = ids.into_iter();

    // Mock producer that just produces IDs from the fixed set
    let mut mock_producer = MockFakeDataProducer::new();
    mock_producer.expect_produce_fake().returning(move |_, _| {
        let id = id_iter
            .next()
            .expect("test should have enough IDs to complete");

        Ok(DataValue::String(id.to_string()))
    });
    let mock_producer = Rc::new(mock_producer);

    let mut mapping = HashMap::new();

    mapping.insert(
        path_key("Posts.[index].Id"),
        Box::new(mock_producer.clone()) as Box<dyn FakeDataProducer>,
    );
    mapping.insert(
        path_key("Posts.[index].Author"),
        Box::new(mock_producer.clone()) as Box<dyn FakeDataProducer>,
    );

    mapping.insert(
        path_key("Posts.[index].Comments.[index].Id"),
        Box::new(mock_producer.clone()) as Box<dyn FakeDataProducer>,
    );
    mapping.insert(
        path_key("Posts.[index].Comments.[index].Parent"),
        Box::new(mock_producer.clone()) as Box<dyn FakeDataProducer>,
    );

    mapping.insert(
        path_key("Posts.[index].Comments.[index].Author"),
        Box::new(mock_producer.clone()) as Box<dyn FakeDataProducer>,
    );

    mapping.insert(
        path_key("Users.[index].Id"),
        Box::new(mock_producer.clone()) as Box<dyn FakeDataProducer>,
    );

    let config = Config {
        mapping,
        default: Some(Box::new(IgnoreFakeData)),
        internal_mapping: true,
        ..Default::default()
    };

    let mut data = UpdateStructureData {
        config,
        output_mapping: Default::default(),
        mapping: Default::default(),
        ctx: Default::default(),
    };

    let input_data: serde_json::Value = serde_json::from_str(sample).unwrap();
    let output_data = process_json_file(input_data, &mut data).unwrap();
    let expected_output_data: serde_json::Value = serde_json::from_str(expected_output).unwrap();

    assert_eq!(output_data, expected_output_data);
}
