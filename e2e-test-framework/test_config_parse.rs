use std::fs::File;
use std::io::BufReader;

fn main() {
    let file = File::open("e2e-test-framework/examples/building_comfort/drasi_server/config.json").unwrap();
    let reader = BufReader::new(file);
    let config: serde_json::Value = serde_json::from_reader(reader).unwrap();
    
    // Extract the local_tests from the config
    if let Some(local_tests) = config["data_store"]["test_repos"][0]["local_tests"].as_array() {
        println\!("Found {} local tests", local_tests.len());
        
        // Try to parse the first test as our TestDefinition
        let test_json = &local_tests[0];
        match serde_json::from_value::<test_data_store::test_repo_storage::models::TestDefinition>(test_json.clone()) {
            Ok(_) => println\!("✓ Successfully parsed TestDefinition from config"),
            Err(e) => println\!("✗ Failed to parse TestDefinition: {}", e)
        }
    }
}
