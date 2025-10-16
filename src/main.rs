use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct CollectionInfo {
    name: String,
    status: Option<String>,
    vectors_count: Option<u64>,
    points_count: Option<u64>,
    indexed_vectors_count: Option<u64>,
    vector_config: Option<Value>,
    error: Option<String>,
}

impl CollectionInfo {
    fn is_healthy(&self) -> bool {
        // A collection is considered healthy if status is "green" and there are no errors
        self.status.as_ref().map(|s| s == "green").unwrap_or(false) && self.error.is_none()
    }
}

#[derive(Parser, Debug)]
#[command(name = "qdrant-collection-cli")]
#[command(about = "Fetch and display Qdrant collections information")]
struct Args {
    /// Filter output by health status: healthy or unhealthy
    #[arg(long, value_name = "TYPE")]
    only: Option<String>,
    
    /// Enable verbose output with additional information
    #[arg(long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    // Validate the --only argument if provided
    if let Some(ref filter) = args.only {
        if filter != "healthy" && filter != "unhealthy" {
            return Err(format!("Invalid value for --only: '{}'. Must be 'healthy' or 'unhealthy'", filter).into());
        }
    }
    
    // Create an HTTP client
    let client = Client::new();
    
    // Make a GET request to the Qdrant collections endpoint
    let url = "http://localhost:6333/collections";
    if args.verbose {
        println!("Calling endpoint: {}", url);
    }
    
    let response = client.get(url).send()?;
    
    // Check if the request was successful
    let status = response.status();
    if args.verbose {
        println!("Response status: {}", status);
    }
    
    // Parse the JSON response
    let body: Value = response.json()?;
    
    // Extract collection names from result.collections[].name
    let collection_names: Vec<String> = body["result"]["collections"]
        .as_array()
        .ok_or("Expected 'result.collections' to be an array")?
        .iter()
        .filter_map(|collection| {
            collection["name"].as_str().map(|s| s.to_string())
        })
        .collect();
    
    if args.verbose {
        println!("\nTotal collections found: {}", collection_names.len());
        println!("Fetching details for each collection...\n");
    }
    
    // Collect all collection information using map
    let collections_info: Vec<CollectionInfo> = collection_names.iter().map(|name| {
        let collection_url = format!("http://localhost:6333/collections/{}", name);
        
        let mut info = CollectionInfo {
            name: name.clone(),
            status: None,
            vectors_count: None,
            points_count: None,
            indexed_vectors_count: None,
            vector_config: None,
            error: None,
        };
        
        match client.get(&collection_url).send() {
            Ok(coll_response) => {
                if coll_response.status().is_success() {
                    match coll_response.json::<Value>() {
                        Ok(coll_data) => {
                            // Extract useful information from the collection details
                            if let Some(result) = coll_data.get("result") {
                                info.status = result.get("status")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                                
                                info.vectors_count = result.get("vectors_count")
                                    .and_then(|v| v.as_u64());
                                
                                info.points_count = result.get("points_count")
                                    .and_then(|v| v.as_u64());
                                
                                info.indexed_vectors_count = result.get("indexed_vectors_count")
                                    .and_then(|v| v.as_u64());
                                
                                if let Some(config) = result.get("config") {
                                    if let Some(params) = config.get("params") {
                                        info.vector_config = params.get("vectors").cloned();
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            info.error = Some(format!("Error parsing collection details: {}", e));
                        }
                    }
                } else {
                    info.error = Some(format!("Failed to get collection details (status: {})", coll_response.status()));
                }
            }
            Err(e) => {
                info.error = Some(format!("Failed to fetch collection details: {}", e));
            }
        }
        
        info
    }).collect();
    
    // Filter collections based on --only flag for display
    let filtered_collections: Vec<&CollectionInfo> = match args.only.as_deref() {
        Some("healthy") => collections_info.iter().filter(|c| c.is_healthy()).collect(),
        Some("unhealthy") => collections_info.iter().filter(|c| !c.is_healthy()).collect(),
        _ => collections_info.iter().collect(),
    };
    
    // Print filtered information
    if args.verbose {        
        if let Some(filter) = &args.only {
            println!("COLLECTION DETAILS (showing only {} collections)", filter);
        } else {
            println!("COLLECTION DETAILS");
        }        
    }
    
    println!("{}", serde_json::to_string_pretty(&filtered_collections)?);
    
    if args.verbose {
        println!();        
        println!("Displayed: {} / {} collections", filtered_collections.len(), collections_info.len());        
    }
    
    Ok(())
}
