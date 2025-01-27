# zspy

A command-line tool for publishing and subscribing to Zenoh topics with protobuf message support.

## Features

- Subscribe to Zenoh topics and display messages
- Publish messages to Zenoh topics
- Support for raw text and protobuf messages
- Configurable publishing rate and repeat count
- List and inspect available message types
- Automatic message type registration

## Installation

1. Make sure you have Rust and Cargo installed
2. Install the protobuf compiler:
   ```bash
   # macOS
   brew install protobuf

   # Ubuntu/Debian
   sudo apt-get install protobuf-compiler

   # Windows (using Chocolatey)
   choco install protoc
   ```
3. Clone and build the project:
   ```bash
   git clone https://github.com/varunkamath/zspy.git
   cd zspy
   cargo build --release
   ```

## Usage

### Subscribe to Messages

```bash
# Subscribe to raw text messages
cargo run -- echo "demo/example/**"

# Subscribe with protobuf decoding
cargo run -- echo "demo/example/**" --type "zspy.ImuMessage"
```

### Publish Messages

```bash
# Publish a single raw text message (default: repeat=1, rate=1.0Hz)
cargo run -- pub "demo/example/test" "Hello Zenoh!"

# Publish a single protobuf message
cargo run -- pub "demo/example/test" \
  --type "zspy.ImuMessage" \
  '{"angular_velocity": {"x": 1.0, "y": 0.0, "z": 0.0}, "timestamp": 1234567890}'

# Publish 10 messages at 2 Hz
cargo run -- pub "demo/example/test" \
  --type "zspy.ImuMessage" \
  '{"angular_velocity": {"x": 1.0, "y": 0.0, "z": 0.0}}' \
  --repeat 10 --rate 2.0

# Publish continuously at 5 Hz (repeat=0 means infinite)
cargo run -- pub "demo/example/test" \
  --type "zspy.ImuMessage" \
  '{"angular_velocity": {"x": 1.0, "y": 0.0, "z": 0.0}}' \
  --repeat 0 --rate 5.0
```

### Message Types

```bash
# List available message types
cargo run -- types list

# Show schema for a message type
cargo run -- types show "zspy.ImuMessage"
```

### List Active Publishers/Subscribers

```bash
# Show session information (discovery feature in development)
cargo run -- list
```

## Adding Custom Protobuf Messages

1. Create your `.proto` file in the `proto/` directory, for example `proto/custom_messages.proto`:
   ```protobuf
   syntax = "proto3";

   package mypackage;

   message CustomMessage {
     string text = 1;
     int32 value = 2;
     repeated double array = 3;
   }
   ```

2. That's it! The build system will automatically:
   - Compile your proto files
   - Generate Rust code
   - Register the message types
   - Add serialization support

Your custom message type is now ready to use:
```bash
# Publish a custom message
cargo run -- pub "demo/example/test" \
  --type "mypackage.CustomMessage" \
  '{"text": "Hello", "value": 42, "array": [1.0, 2.0, 3.0]}'

# Subscribe to custom messages
cargo run -- echo "demo/example/**" --type "mypackage.CustomMessage"
```

The message registry system automatically handles:
- JSON to protobuf conversion
- Protobuf to JSON conversion
- Message type validation
- Schema information (in development)
- Message type listing

## License

WTFPL License - Do whatever you want with it.