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
Traditional terminal-based interactive chat.

```bash
# Interactive mode (default)
cargo run

# With specific region
cargo run -- --region us-west-2
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
- 📄 **Document Support**: Upload and analyze multiple documents (TXT, MD, JSON, XML, CSV, HTML, DOCX)
- 💬 **Conversation Memory**: The app remembers your conversation context
- 📊 **Token Usage**: See input/output token usage for each request
- 🌍 **Multi-Region**: Support for different AWS regions
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
| DOCX | `.docx` | ✅ Supported | Word documents |
| PDF | `.pdf` | 🚧 Binary info only | Reports, documents |

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

#### Specify AWS Region
```bash
cargo run -- --region us-west-2
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

```

## Document Processing

### What Happens After Uploading a DOCX File

When you upload a DOCX file to the application, here's what happens:

1. **File Detection**: The application detects the `.docx` extension
2. **Text Extraction**: Using the `dotext` crate, the application extracts all readable text from the DOCX file, including:
   - Paragraph text
   - Table content
   - Headers and footers
   - Text from various document elements
3. **Content Preparation**: The extracted text is formatted and prepared for Claude
4. **AI Analysis**: The text content is sent to Claude Haiku 3.5 along with your question

### Example DOCX Processing

```bash
💬 You: file my_report.docx
📎 Added file: my_report.docx

💬 You: What are the main conclusions in this document?
📎 Loaded document: my_report.docx
🤖 Claude: 📊 Tokens used - Input: 2150, Output: 280

Based on the document you've shared, here are the main conclusions:

1. **Executive Summary**: The report concludes that...
2. **Key Findings**: The analysis shows...
3. **Recommendations**: The document suggests...

[Claude provides detailed analysis of the extracted DOCX content]
```

### Supported Content Types

The DOCX text extraction supports:
- ✅ **Paragraphs**: All text paragraphs and formatting
- ✅ **Tables**: Content from table cells
- ✅ **Headers/Footers**: Document headers and footers
- ✅ **Lists**: Bulleted and numbered lists
- ⚠️ **Images**: Image descriptions may not be extracted
- ⚠️ **Complex Formatting**: Some advanced formatting may be simplified

### Technical Details

- **Library Used**: `dotext` crate for reliable text extraction
- **File Size Limits**: Large DOCX files may increase token usage significantly
- **Error Handling**: If text extraction fails, the application will show file info instead
- **Performance**: Text extraction is fast and efficient for most document sizes

## Document Processing Notes

- Documents are automatically processed and sent to Claude
- File types are detected based on file extensions
- Large documents may increase token usage significantly
- Claude can analyze text content, extract data, compare documents, and answer questions about the content