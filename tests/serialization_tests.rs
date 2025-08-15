#[cfg(feature = "serialize")]
mod serialization_tests {
    use serde::{Deserialize, Serialize};
    use thread_share::ThreadShare;

    #[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
    struct TestData {
        id: u32,
        name: String,
        values: Vec<i32>,
    }

    #[test]
    fn test_json_serialization() {
        let data = ThreadShare::new(TestData {
            id: 1,
            name: "test".to_string(),
            values: vec![1, 2, 3],
        });

        // Test serialization
        let json = data.to_json().expect("Failed to serialize");
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"values\":[1,2,3]"));

        // Test deserialization
        let new_json = r#"{"id":2,"name":"updated","values":[4,5,6]}"#;
        data.from_json(new_json).expect("Failed to deserialize");

        let updated = data.get();
        assert_eq!(updated.id, 2);
        assert_eq!(updated.name, "updated");
        assert_eq!(updated.values, vec![4, 5, 6]);
    }

    #[test]
    fn test_primitive_serialization() {
        let counter = ThreadShare::new(42);

        let json = counter.to_json().expect("Failed to serialize");
        assert_eq!(json, "42");

        counter.from_json("100").expect("Failed to deserialize");
        assert_eq!(counter.get(), 100);
    }

    #[test]
    fn test_vector_serialization() {
        let data = ThreadShare::new(vec![1, 2, 3]);

        let json = data.to_json().expect("Failed to serialize");
        assert_eq!(json, "[1,2,3]");

        data.from_json("[4,5,6]").expect("Failed to deserialize");
        assert_eq!(data.get(), vec![4, 5, 6]);
    }

    #[test]
    fn test_string_serialization() {
        let data = ThreadShare::new("Hello, World!".to_string());

        let json = data.to_json().expect("Failed to serialize");
        assert_eq!(json, "\"Hello, World!\"");

        data.from_json("\"Updated String\"")
            .expect("Failed to deserialize");
        assert_eq!(data.get(), "Updated String");
    }
}

#[cfg(not(feature = "serialize"))]
mod serialization_tests {
    #[test]
    fn test_serialization_feature_disabled() {
        // This test runs when serialize feature is disabled
        // It ensures the library compiles without serialization support
        assert!(true);
    }
}
