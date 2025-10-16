# Qdrant Collection Inspector

A Rust command-line tool for fetching and displaying information about Qdrant vector database collections.

## Features

- Fetches all collections from a Qdrant instance
- Retrieves detailed information for each collection including:
  - Collection status
  - Vector count
  - Points count
  - Indexed vectors count
  - Vector configuration
- Filter collections by health status (healthy/unhealthy)
- JSON output for easy integration with other tools
- Verbose mode for debugging and interactive use

## Installation

### Prerequisites

- Rust 1.70 or later
- A running Qdrant instance (default: `http://localhost:6333`)

### Build from source

```bash
git clone <repository-url>
cd qdrant-collection-cli
cargo build --release
```

The compiled binary will be available at `target/release/qdrant-collection-cli`.

## Usage

### Basic usage

Show all collections (JSON output only):

```bash
cargo run
```

Or with the compiled binary:

```bash
./target/release/qdrant-collection-cli
```

### Verbose output

Show collections with additional information:

```bash
cargo run -- --verbose
```

### Filter by health status

Show only healthy collections:

```bash
cargo run -- --only=healthy
```

Show only unhealthy collections:

```bash
cargo run -- --only=unhealthy
```

### Combined options

Show only unhealthy collections with verbose output:

```bash
cargo run -- --only=unhealthy --verbose
```

## Command-line options

| Option          | Description                                              |
| --------------- | -------------------------------------------------------- |
| `--only <TYPE>` | Filter output by health status: `healthy` or `unhealthy` |
| `--verbose`     | Enable verbose output with additional information        |
| `-h, --help`    | Print help information                                   |

## Health status criteria

A collection is considered **healthy** if:
- Status is "green"
- No errors occurred during fetching

A collection is considered **unhealthy** if:
- Status is not "green" (e.g., "yellow", "red")
- An error occurred while fetching collection details

## Output format

### Default (JSON only)

```json
[
  {
    "name": "my_collection",
    "status": "green",
    "vectors_count": 1000,
    "points_count": 1000,
    "indexed_vectors_count": 1000,
    "vector_config": {
      "size": 384,
      "distance": "Cosine"
    },
    "error": null
  }
]
```

### Verbose mode

```
Calling endpoint: http://localhost:6333/collections
Response status: 200 OK

Total collections found: 3
Fetching details for each collection...

============================================================
COLLECTION DETAILS
============================================================

[
  {
    "name": "my_collection",
    "status": "green",
    ...
  }
]

============================================================
Displayed: 3 / 3 collections
============================================================
```

## Configuration

The tool connects to Qdrant at `http://localhost:6333` by default. To use a different endpoint, you'll need to modify the `url` variable in `src/main.rs`.

## Dependencies

- `reqwest` - HTTP client for making API requests
- `serde` & `serde_json` - JSON serialization/deserialization
- `clap` - Command-line argument parsing

## Error handling

The tool handles errors gracefully:
- Connection errors are captured per collection
- Failed collections are still included in the output with error messages
- The tool continues processing remaining collections even if some fail

## Examples

### Get only names of unhealthy collections

```bash
cargo run -- --only=unhealthy | jq -r '.[].name'
```

This will output just the collection names, one per line:
```
broken_collection_1
broken_collection_2
```

### Pipe to jq for further processing

```bash
cargo run | jq '.[] | select(.vectors_count > 1000)'
```

### Save output to file

```bash
cargo run > collections.json
```

### Check for unhealthy collections in a script

```bash
if cargo run -- --only=unhealthy | jq -e 'length > 0' > /dev/null; then
  echo "Found unhealthy collections!"
  exit 1
fi
```
