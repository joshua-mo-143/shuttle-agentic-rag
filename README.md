Here's a suggested README.md for your Rust Axum server application:

```markdown
# Rust Axum Server with OpenAI and Qdrant Integration

This Rust-based server application leverages Axum for creating HTTP-based services, integrates with OpenAI's APIs, and uses Qdrant for vector database functionalities. It is designed to process and respond to prompts using AI and store embeddings in Qdrant.

## Features

- **Axum Server**: Utilizes Axum for handling HTTP requests efficiently with routing and middleware support.
- **OpenAI Integration**: Connects with OpenAI to generate responses based on prompts.
- **Qdrant Integration**: Uses Qdrant to store and retrieve document embeddings, allowing semantic search capabilities.
- **Middleware Logging**: Includes middleware for logging requests and responses to help in debugging and monitoring the server.

## Prerequisites

Before you can run the server, you will need:
- Rust and Cargo installed on your machine.
- Access to OpenAI API key.
- Access to Qdrant cloud service with a valid URL and API token.

## Setup

1. **Clone the repository**:
   ```bash
   git clone [your-repository-url]
   cd [repository-name]
   ```

2. **Environment Variables**:
   Set up the necessary environment variables or use a `.env` file to store them:
   ```plaintext
   OPENAI_API_KEY="your_openai_api_key_here"
   QDRANT_URL="your_qdrant_url_here"
   QDRANT_API_TOKEN="your_qdrant_api_token_here"
   ```

3. **Build the Application**:
   ```bash
   cargo build
   ```

4. **Run the Server**:
   ```bash
   cargo run
   ```

## API Endpoints

- `GET /`: A simple hello world endpoint.
- `POST /prompt`: Takes a JSON payload with a `prompt` field and returns a response from the AI model.

## Architecture

This application is structured into several modules:
- `main.rs`: Contains the server setup, route configuration, and main function.
- `agents.rs`: Defines the behavior of the AI agents, including interactions with OpenAI and Qdrant.
- `files.rs`: Handles file operations, particularly reading and parsing for embedding purposes.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or create issues for bugs and feature requests.

## License

Specify your license here or indicate if it's proprietary.
```

This README provides a general overview of your application, how to set it up, and run it, alongside the architecture and contribution guide. Adjust the repository URL, repository name, and other specific details as necessary.
