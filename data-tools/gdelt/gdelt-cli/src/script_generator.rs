// Copyright 2025 The Drasi Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::path::{Path, PathBuf};
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs::{create_dir_all, File};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

// Re-create the structs from test-data-store/src/scripts/mod.rs to match the expected format

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceChangeEvent {
    pub op: String,
    #[serde(rename = "reactivatorStart_ns")]
    pub reactivator_start_ns: u64,
    #[serde(rename = "reactivatorEnd_ns")]
    pub reactivator_end_ns: u64,
    pub payload: SourceChangeEventPayload,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceChangeEventPayload {
    pub source: SourceChangeEventSourceInfo,
    pub before: Value,
    pub after: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceChangeEventSourceInfo {
    pub db: String,
    pub table: String,
    pub ts_ns: u64,
    pub lsn: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum BootstrapScriptRecord {
    Comment(CommentRecord),
    Header(BootstrapHeaderRecord),
    Label(LabelRecord),
    Node(NodeRecord),
    Relation(RelationRecord),
    Finish(BootstrapFinishRecord),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ChangeScriptRecord {
    Comment(CommentRecord),
    Header(ChangeHeaderRecord),
    Label(LabelRecord),
    PauseCommand(PauseCommandRecord),
    SourceChange(SourceChangeRecord),
    Finish(ChangeFinishRecord),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommentRecord {
    pub comment: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BootstrapHeaderRecord {
    pub start_time: DateTime<FixedOffset>,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeHeaderRecord {
    pub start_time: DateTime<FixedOffset>,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LabelRecord {
    #[serde(default)]
    pub offset_ns: u64,
    pub label: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PauseCommandRecord {
    #[serde(default)]
    pub offset_ns: u64,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BootstrapFinishRecord {
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeFinishRecord {
    #[serde(default)]
    pub offset_ns: u64,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceChangeRecord {
    #[serde(default)]
    pub offset_ns: u64,
    pub source_change_event: SourceChangeEvent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeRecord {
    pub id: String,
    pub labels: Vec<String>,
    #[serde(default)]
    pub properties: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationRecord {
    pub id: String,
    pub labels: Vec<String>,
    pub start_id: String,
    pub start_label: Option<String>,
    pub end_id: String,
    pub end_label: Option<String>,
    #[serde(default)]
    pub properties: Value,
}

pub struct ScriptGenerator {
    output_path: PathBuf,
}

impl ScriptGenerator {
    pub fn new(output_path: PathBuf) -> Self {
        Self { output_path }
    }

    pub async fn generate_bootstrap_scripts(
        &self,
        source_files: Vec<PathBuf>,
        source_type: &str,
    ) -> anyhow::Result<()> {
        let bootstrap_path = self.output_path.join("bootstrap_scripts");
        create_dir_all(&bootstrap_path).await?;

        // Create header record
        let header = BootstrapScriptRecord::Header(BootstrapHeaderRecord {
            start_time: Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()),
            description: format!("GDELT {} bootstrap data", source_type),
        });

        // Write header to the first file
        let header_file = bootstrap_path.join(format!("{}_header.jsonl", source_type));
        let mut file = File::create(&header_file).await?;
        file.write_all(format!("{}\n", serde_json::to_string(&header)?).as_bytes()).await?;

        // Process each source file and convert to node records
        for (idx, source_file) in source_files.iter().enumerate() {
            let output_file = bootstrap_path.join(format!("{}_{:05}.jsonl", source_type, idx));
            self.convert_to_bootstrap_nodes(&source_file, &output_file, source_type).await?;
        }

        // Create finish record
        let finish = BootstrapScriptRecord::Finish(BootstrapFinishRecord {
            description: format!("Completed {} bootstrap data", source_type),
        });

        let finish_file = bootstrap_path.join(format!("{}_finish.jsonl", source_type));
        let mut file = File::create(&finish_file).await?;
        file.write_all(format!("{}\n", serde_json::to_string(&finish)?).as_bytes()).await?;

        Ok(())
    }

    pub async fn generate_change_scripts(
        &self,
        source_files: Vec<PathBuf>,
        source_type: &str,
        start_time: DateTime<FixedOffset>,
    ) -> anyhow::Result<()> {
        let changes_path = self.output_path.join("source_change_scripts");
        create_dir_all(&changes_path).await?;

        // Create header record
        let header = ChangeScriptRecord::Header(ChangeHeaderRecord {
            start_time,
            description: format!("GDELT {} change data", source_type),
        });

        // Write header to the first file
        let header_file = changes_path.join("source_change_scripts_00000.jsonl");
        let mut file = File::create(&header_file).await?;
        file.write_all(format!("{}\n", serde_json::to_string(&header)?).as_bytes()).await?;

        // Process source files and convert to change events
        let mut file_counter = 1;
        let mut current_offset_ns = 0u64;

        for source_file in source_files {
            let output_file = changes_path.join(format!("source_change_scripts_{:05}.jsonl", file_counter));
            current_offset_ns = self.convert_to_change_events(
                &source_file,
                &output_file,
                source_type,
                current_offset_ns,
            ).await?;
            file_counter += 1;
        }

        // Create finish record
        let finish = ChangeScriptRecord::Finish(ChangeFinishRecord {
            offset_ns: current_offset_ns,
            description: format!("Completed {} change data", source_type),
        });

        let finish_file = changes_path.join(format!("source_change_scripts_{:05}.jsonl", file_counter));
        let mut file = File::create(&finish_file).await?;
        file.write_all(format!("{}\n", serde_json::to_string(&finish)?).as_bytes()).await?;

        Ok(())
    }

    async fn convert_to_bootstrap_nodes(
        &self,
        source_file: &Path,
        output_file: &Path,
        source_type: &str,
    ) -> anyhow::Result<()> {
        let file = File::open(source_file).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut output = File::create(output_file).await?;

        // Skip header line if CSV
        if source_file.extension().and_then(|s| s.to_str()) == Some("CSV") {
            let _ = lines.next_line().await?;
        }

        while let Some(line) = lines.next_line().await? {
            // Parse the line based on source_type (event, graph, mention)
            // This is a simplified example - you'll need to implement proper CSV parsing
            // based on GDELT format specifications
            
            let node = match source_type {
                "event" => self.parse_event_to_node(&line)?,
                "graph" => self.parse_graph_to_node(&line)?,
                "mention" => self.parse_mention_to_node(&line)?,
                _ => continue,
            };

            let record = BootstrapScriptRecord::Node(node);
            output.write_all(format!("{}\n", serde_json::to_string(&record)?).as_bytes()).await?;
        }

        Ok(())
    }

    async fn convert_to_change_events(
        &self,
        source_file: &Path,
        output_file: &Path,
        source_type: &str,
        start_offset_ns: u64,
    ) -> anyhow::Result<u64> {
        let file = File::open(source_file).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut output = File::create(output_file).await?;
        let mut current_offset_ns = start_offset_ns;

        // Skip header line if CSV
        if source_file.extension().and_then(|s| s.to_str()) == Some("CSV") {
            let _ = lines.next_line().await?;
        }

        while let Some(line) = lines.next_line().await? {
            // Parse the line and create a change event
            let (before, after) = match source_type {
                "event" => self.parse_event_change(&line)?,
                "graph" => self.parse_graph_change(&line)?,
                "mention" => self.parse_mention_change(&line)?,
                _ => continue,
            };

            let source_change_event = SourceChangeEvent {
                op: if before.is_null() { "c" } else { "u" }.to_string(),
                reactivator_start_ns: current_offset_ns,
                reactivator_end_ns: current_offset_ns + 1000000, // 1ms later
                payload: SourceChangeEventPayload {
                    source: SourceChangeEventSourceInfo {
                        db: "gdelt".to_string(),
                        table: source_type.to_string(),
                        ts_ns: current_offset_ns,
                        lsn: current_offset_ns / 1000000, // Convert to ms for LSN
                    },
                    before,
                    after,
                },
            };

            let record = ChangeScriptRecord::SourceChange(SourceChangeRecord {
                offset_ns: current_offset_ns,
                source_change_event,
            });

            output.write_all(format!("{}\n", serde_json::to_string(&record)?).as_bytes()).await?;
            
            current_offset_ns += 15 * 60 * 1_000_000_000; // 15 minutes in nanoseconds
        }

        Ok(current_offset_ns)
    }

    // Placeholder parsing methods - these need to be implemented based on GDELT format
    fn parse_event_to_node(&self, _line: &str) -> anyhow::Result<NodeRecord> {
        // TODO: Implement actual GDELT event parsing
        Ok(NodeRecord {
            id: "event_placeholder".to_string(),
            labels: vec!["Event".to_string()],
            properties: serde_json::json!({}),
        })
    }

    fn parse_graph_to_node(&self, _line: &str) -> anyhow::Result<NodeRecord> {
        // TODO: Implement actual GDELT graph parsing
        Ok(NodeRecord {
            id: "graph_placeholder".to_string(),
            labels: vec!["Graph".to_string()],
            properties: serde_json::json!({}),
        })
    }

    fn parse_mention_to_node(&self, _line: &str) -> anyhow::Result<NodeRecord> {
        // TODO: Implement actual GDELT mention parsing
        Ok(NodeRecord {
            id: "mention_placeholder".to_string(),
            labels: vec!["Mention".to_string()],
            properties: serde_json::json!({}),
        })
    }

    fn parse_event_change(&self, _line: &str) -> anyhow::Result<(Value, Value)> {
        // TODO: Implement actual GDELT event change parsing
        Ok((Value::Null, serde_json::json!({})))
    }

    fn parse_graph_change(&self, _line: &str) -> anyhow::Result<(Value, Value)> {
        // TODO: Implement actual GDELT graph change parsing
        Ok((Value::Null, serde_json::json!({})))
    }

    fn parse_mention_change(&self, _line: &str) -> anyhow::Result<(Value, Value)> {
        // TODO: Implement actual GDELT mention change parsing
        Ok((Value::Null, serde_json::json!({})))
    }
}