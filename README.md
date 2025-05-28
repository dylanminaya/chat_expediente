# Chat Expediente - Claude Haiku 3.5 Chat

A Rust application to chat with Claude Haiku 3.5 via AWS Bedrock with support for multiple document types and interfaces.

## 🚀 Interface Options

### 1. 🌐 **Web Interface** (Recommended)
Modern, responsive web UI with drag-and-drop file upload.

```bash
cargo run -- --web --port 3000
```

Then open http://localhost:3000 in your browser.

**Features:**
- 📱 Responsive design (works on mobile)
- 🖱️ Drag & drop file upload
- 💬 Real-time chat interface
- 📊 Token usage display
- 🎨 Beautiful modern UI

### 2. 💻 **Command Line Interface**
Traditional terminal-based chat.

```bash
# Interactive mode (default)
cargo run

# Single message
cargo run -- --message "Hello Claude!"

# With documents
cargo run -- --message "Analyze this" --files document.pdf
```

### 3. 🔌 **API Integration**
Use the web server as an API backend for your own applications.

```bash
# Start API server
cargo run -- --web --port 3000

# API Endpoints:
# POST /api/chat - Send messages
# POST /api/upload - Upload files
# GET /api/conversations?id=<id> - Get conversation history
```

## Features

- 🤖 **Interactive Chat**: Have ongoing conversations with Claude Haiku 3.5
- 📄 **Document Support**: Upload and analyze multiple documents (PDF, TXT, MD, JSON, XML, CSV, HTML, DOCX)
- 💬 **Conversation Memory**: The app remembers your conversation context
- 📊 **Token Usage**: See input/output token usage for each request
- 🌍 **Multi-Region**: Support for different AWS regions
- 🎯 **Single Message**: Send one-off messages with or without documents
- 🧹 **History Management**: Clear or view conversation history
- 🌐 **Web Interface**: Modern browser-based UI
- 🔌 **API Endpoints**: RESTful API for integration

## Prerequisites

1. **AWS Account**: You need an AWS account with access to Amazon Bedrock
2. **AWS CLI**: Install and configure AWS CLI with your credentials
3. **Bedrock Access**: Ensure you have access to Claude Haiku 3.5 in your AWS region
4. **Rust**: Make sure you have Rust installed

## Supported Document Types

| Format | Extension | Status | Use Case |
|--------|-----------|--------|----------|
| Text | `.txt`, `.md` | ✅ Supported | Documentation, notes |
| JSON | `.json` | ✅ Supported | Structured data |
| CSV | `.csv` | ✅ Supported | Tabular data |
| HTML | `.html`, `.htm` | ✅ Supported | Web content |
| XML | `.xml` | ✅ Supported | Structured markup |
| PDF | `.pdf` | 🚧 Text extraction | Reports, documents |
| DOCX | `.docx` | 📋 Planned | Office documents |

## Installation and Usage

### Build the application

```bash
cd chat_expediente
cargo build --release
```

### 🌐 Web Interface Usage

#### Start Web Server
```bash
cargo run -- --web --port 3000
```

#### Open Browser
Navigate to http://localhost:3000

#### Upload Documents
- **Drag & Drop**: Drag files directly onto the upload area
- **Click Upload**: Click the upload area to select files
- **Multiple Files**: Upload multiple documents at once

#### Chat Features
- Type messages in the text area
- Press Enter to send (Shift+Enter for new lines)
- View conversation history
- See token usage for each response

### 💻 Command Line Usage

#### Interactive Chat Mode (Default)
```bash
cargo run
```

#### Interactive Chat with Document Upload
```bash
# In interactive mode, use these commands:
# file <path>           - Add a single document
# files <path1> <path2> - Add multiple documents
# Then type your message to send text + documents together
```

#### Single Message Mode
```bash
cargo run -- --message "Hello, Claude! How are you today?"
```

#### Single Message with Documents
```bash
cargo run -- --message "Analyze these documents" --files document1.pdf document2.txt
```

#### Specify AWS Region
```bash
cargo run -- --region us-west-2 --web
```

### 🔌 API Integration

#### Start API Server
```bash
cargo run -- --web --port 3000
```

#### API Endpoints

**Send Chat Message**
```bash
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Hello Claude!",
    "conversation_id": "optional-uuid",
    "files": ["path/to/uploaded/file.txt"]
  }'
```

**Upload Files**
```bash
curl -X POST http://localhost:3000/api/upload \
  -F "files=@document.pdf" \
  -F "files=@data.json"
```

**Get Conversation History**
```bash
curl "http://localhost:3000/api/conversations?id=conversation-uuid"
```

### Command Line Options

- `--web, -w`: Start web server mode
- `--port, -p`: Web server port (default: 3000)
- `--region, -r`: AWS region (default: us-east-1)
- `--message, -m`: Send a single message and exit
- `--interactive, -i`: Enable interactive chat mode (default behavior)
- `--files, -f`: Attach documents to your message (can be used multiple times)

## Interactive Commands (CLI Mode)

When in interactive mode, you can use these special commands:

- `quit`, `exit`, or `bye`: End the conversation
- `clear`: Clear the conversation history and pending files
- `history`: Show the conversation history
- `file <path>`: Add a document to your next message
- `files <path1> <path2> ...`: Add multiple documents to your next message
- Just type your message and press Enter to chat

## Example Usage

### 🌐 Web Interface Example

1. **Start the server:**
   ```bash
   cargo run -- --web
   ```

2. **Open browser:** http://localhost:3000

3. **Upload documents:** Drag PDF, TXT, or JSON files to the upload area

4. **Ask questions:** "What are the key findings in these documents?"

5. **Get analysis:** Claude will analyze all uploaded documents and provide insights

### 💻 CLI Example

```bash
$ cargo run

🚀 Initializing connection to Claude Haiku 3.5...
🌍 Region: us-east-1
✅ Connected successfully!
🤖 Claude Haiku 3.5 Chat
Type 'quit', 'exit', or 'bye' to end the conversation
Type 'clear' to clear the conversation history
Type 'history' to see the conversation history
Type 'file <path>' to attach a document to your next message
Type 'files <path1> <path2> ...' to attach multiple documents
==================================================

💬 You: Hello! Can you help me analyze some documents?
🤖 Claude: 📊 Tokens used - Input: 15, Output: 45
Hello! I'd be happy to help you analyze documents. You can upload documents using the 'file' or 'files' commands, then ask me questions about them. What kind of analysis are you looking for?

💬 You: file sample_report.md
📎 Added file: sample_report.md

💬 You: What are the key findings in this report?
📎 Loaded document: sample_report.md
🤖 Claude: 📊 Tokens used - Input: 1250, Output: 180
Based on the report you've shared, here are the key findings:
[Analysis of the Markdown content...]
```

### 🔌 API Integration Example

```javascript
// JavaScript example for web integration
async function chatWithClaude(message, files = []) {
    const response = await fetch('/api/chat', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            message: message,
            files: files
        })
    });
    
    const data = await response.json();
    return data.response;
}

// Upload files first
const formData = new FormData();
formData.append('files', fileInput.files[0]);

const uploadResponse = await fetch('/api/upload', {
    method: 'POST',
    body: formData
});

const filePaths = await uploadResponse.json();

// Then chat with uploaded files
const response = await chatWithClaude(
    "Analyze this document", 
    filePaths
);
```

## Document Processing

- Documents are automatically processed and sent to Claude
- File types are detected based on file extensions
- Large documents may increase token usage significantly
- Claude can analyze text content, extract data, compare documents, and answer questions about the content

## AWS Setup

### 1. Configure AWS Credentials

Make sure your AWS credentials are configured. You can do this in several ways:

```bash
# Option 1: AWS CLI
aws configure

# Option 2: Environment variables
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_DEFAULT_REGION=us-east-1

# Option 3: AWS credentials file (~/.aws/credentials)
[default]
aws_access_key_id = your_access_key
aws_secret_access_key = your_secret_key
```

### 2. Enable Claude Haiku 3.5 in Bedrock

1. Go to the AWS Bedrock console
2. Navigate to "Model access" in the left sidebar
3. Click "Request model access"
4. Find "Claude 3.5 Haiku" and request access
5. Wait for approval (usually instant for most accounts)

## Interface Comparison

| Feature | Web Interface | CLI Interface | API Integration |
|---------|---------------|---------------|-----------------|
| **Ease of Use** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| **File Upload** | Drag & Drop | Command-based | Programmatic |
| **Mobile Support** | ✅ Yes | ❌ No | ✅ Yes |
| **Visual Appeal** | ⭐⭐⭐⭐⭐ | ⭐⭐ | N/A |
| **Automation** | ❌ No | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Integration** | ❌ No | ❌ No | ⭐⭐⭐⭐⭐ |
| **Performance** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

## Troubleshooting

### Common Issues

1. **Authentication Error**: Make sure your AWS credentials are properly configured
2. **Model Access Error**: Ensure you have requested access to Claude Haiku 3.5 in Bedrock
3. **Region Error**: Make sure Claude Haiku 3.5 is available in your specified region
4. **Network Error**: Check your internet connection and AWS service status
5. **File Not Found**: Ensure document paths are correct and files exist
6. **Large File Error**: Very large documents may exceed token limits
7. **Web Server Not Starting**: Check if port is already in use
8. **Browser Not Loading**: Ensure you're using the correct URL and port

### Supported Regions

Claude Haiku 3.5 is available in these AWS regions:
- us-east-1 (N. Virginia)
- us-west-2 (Oregon)
- eu-west-1 (Ireland)
- ap-southeast-1 (Singapore)
- ap-northeast-1 (Tokyo)

## Model Information

- **Model ID**: `us.anthropic.claude-3-5-haiku-20241022-v1:0` (inference profile)
- **Max Tokens**: 4096
- **Temperature**: 0.7
- **API Version**: bedrock-2023-05-31
- **Document Support**: PDF, TXT, MD, JSON, XML, CSV, HTML, DOCX, and more

## Development

### Adding New Features

1. **New Document Types**: Modify `load_document_as_text()` function
2. **UI Improvements**: Edit `static/index.html`
3. **API Endpoints**: Add routes in `web_server.rs`
4. **CLI Commands**: Update interactive chat logic

### Building for Production

```bash
cargo build --release
./target/release/chat_expediente --web --port 8080
```

## License

This project is open source. Feel free to modify and use as needed. 