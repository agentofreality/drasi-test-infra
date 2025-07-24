// Example showing how to use the unified HandlerRecord structure

use std::collections::HashMap;
use test_run_host::queries::output_handler_message::{
    HandlerError, HandlerPayload, HandlerRecord, HandlerType, OutputHandlerMessage,
};
use test_run_host::queries::result_stream_record::{ChangeEvent, QueryResultRecord};

fn main() {
    // Example 1: Creating a HandlerRecord for a result stream
    let result_stream_record = HandlerRecord {
        id: "query-123".to_string(),
        sequence: 1,
        created_time_ns: 1000,
        processed_time_ns: 2000,
        traceparent: Some("00-trace-01-1".to_string()),
        tracestate: None,
        payload: HandlerPayload::ResultStream {
            query_result: QueryResultRecord::Change(ChangeEvent {
                // ... change event fields
            }),
        },
    };

    // Example 2: Creating a HandlerRecord for a reaction invocation
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let reaction_record = HandlerRecord {
        id: "reaction-456".to_string(),
        sequence: 2,
        created_time_ns: 3000,
        processed_time_ns: 3000,
        traceparent: Some("00-trace-02-1".to_string()),
        tracestate: None,
        payload: HandlerPayload::ReactionInvocation {
            reaction_type: "http".to_string(),
            query_id: "query-123".to_string(),
            request_method: "POST".to_string(),
            request_path: "/reaction".to_string(),
            request_body: serde_json::json!({"status": "ok"}),
            headers,
        },
    };

    // Example 3: Processing unified messages
    let messages = vec![
        OutputHandlerMessage::Record(result_stream_record),
        OutputHandlerMessage::Record(reaction_record),
        OutputHandlerMessage::HandlerStopping {
            handler_type: HandlerType::ResultStream,
        },
        OutputHandlerMessage::Error {
            handler_type: HandlerType::Reaction,
            error: HandlerError::ConversionError,
        },
    ];

    for message in messages {
        match message {
            OutputHandlerMessage::Record(record) => {
                println!(
                    "Processing record: id={}, seq={}",
                    record.id, record.sequence
                );
                match record.payload {
                    HandlerPayload::ResultStream { query_result } => {
                        println!("  Result stream query result: {:?}", query_result);
                    }
                    HandlerPayload::ReactionInvocation { query_id, .. } => {
                        println!("  Reaction invocation for query: {}", query_id);
                    }
                }
            }
            OutputHandlerMessage::HandlerStopping { handler_type } => {
                println!("Handler stopping: {:?}", handler_type);
            }
            OutputHandlerMessage::Error {
                handler_type,
                error,
            } => {
                println!("Handler error: {:?} - {:?}", handler_type, error);
            }
        }
    }
}
