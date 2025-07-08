## Rust Application Logging with AWS CloudWatch Integration

- This guide outlines the implementation and configuration of logging in a Rust application, designed to integrate with
  AWS CloudWatch for efficient log management and monitoring.

### System Overview

- Our application's logging mechanism is streamlined through a custom macro and is configured to send logs to AWS
  CloudWatch. The key components of this system include:

- **`src/utils/logs.rs`**: Contains the core logic for logging and AWS CloudWatch integration.
- **`main.rs`**: Defines the `log!` macro for application-wide use.

### Environment Configuration

- To ensure secure and correct operation, the application relies on several environment variables:

```sh
AWS_REGION=ap-south-1
CLOUDWATCH_AWS_ACCESS_KEY=your_access_key_here
CLOUDWATCH_AWS_SECRET_KEY=your_secret_key_here
CLOUDWATCH_AWS_REGION=ap-south-1
AWS_LOG_GROUP=RustFrameWork
LOG_TO_CLOUDWATCH=false/true
```

### Logging Levels and Streams

- The application categorizes logs into different levels, each corresponding to a specific AWS CloudWatch log stream:

```sh
Level::Error => LogStream::ServerErrorResponses,
Level::Warn => LogStream::ClientErrorResponses,
Level::Info => LogStream::InformationalResponses,
Level::Debug => LogStream::DebuggingResponses,
Level::Trace => LogStream::TraceResponses,
```

### log! Macro Usage

```sh
log!(Level::Info, "Application starting up.");
log!(Level::Warn, "Missing configuration detected.");
log!(Level::Error, "Database connection failed.");

log!(Level::Info, &format!("msg {}", veriable));

```

> **Logs must be used in *`services`***