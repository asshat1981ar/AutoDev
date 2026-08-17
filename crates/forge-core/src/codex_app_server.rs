//! Transport adapters for the local `codex app-server` process.
//!
//! The supported production path is stdio JSONL. The app-server owns ChatGPT
//! credentials; this transport only exchanges protocol messages over local
//! process pipes.

use std::{
    collections::VecDeque,
    ffi::OsStr,
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use serde_json::{json, Value};

use crate::{CodexRpcTransport, CodexSubscriptionError};

pub struct JsonlCodexTransport<R: BufRead, W: Write> {
    reader: R,
    writer: W,
    next_id: u64,
    notifications: VecDeque<Value>,
}

impl<R: BufRead, W: Write> JsonlCodexTransport<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader,
            writer,
            next_id: 0,
            notifications: VecDeque::new(),
        }
    }

    pub fn pop_notification(&mut self) -> Option<Value> {
        self.notifications.pop_front()
    }

    fn write_message(&mut self, message: &Value) -> Result<(), CodexSubscriptionError> {
        serde_json::to_writer(&mut self.writer, message)
            .map_err(|error| CodexSubscriptionError::Protocol(error.to_string()))?;
        self.writer
            .write_all(b"\n")
            .and_then(|_| self.writer.flush())
            .map_err(|error| CodexSubscriptionError::Protocol(error.to_string()))
    }

    fn read_response(&mut self, expected_id: u64) -> Result<Value, CodexSubscriptionError> {
        loop {
            let mut line = String::new();
            let bytes = self
                .reader
                .read_line(&mut line)
                .map_err(|error| CodexSubscriptionError::Protocol(error.to_string()))?;
            if bytes == 0 {
                return Err(CodexSubscriptionError::Protocol(
                    "codex app-server closed the JSONL stream".into(),
                ));
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let message: Value = serde_json::from_str(trimmed)
                .map_err(|error| CodexSubscriptionError::Protocol(error.to_string()))?;

            if message.get("id").is_none() && message.get("method").is_some() {
                self.notifications.push_back(message);
                continue;
            }

            let response_id = message.get("id").and_then(Value::as_u64).ok_or_else(|| {
                CodexSubscriptionError::Protocol("response is missing a numeric id".into())
            })?;
            if response_id != expected_id {
                return Err(CodexSubscriptionError::Protocol(format!(
                    "unexpected response id {response_id}; expected {expected_id}"
                )));
            }

            if let Some(error) = message.get("error") {
                let code = error.get("code").and_then(Value::as_i64);
                let message_text = error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown app-server error");
                return Err(CodexSubscriptionError::Protocol(match code {
                    Some(code) => format!("app-server error {code}: {message_text}"),
                    None => format!("app-server error: {message_text}"),
                }));
            }

            return message.get("result").cloned().ok_or_else(|| {
                CodexSubscriptionError::Protocol("response is missing result".into())
            });
        }
    }
}

impl<R: BufRead, W: Write> CodexRpcTransport for JsonlCodexTransport<R, W> {
    fn request(&mut self, method: &str, params: Value) -> Result<Value, CodexSubscriptionError> {
        let id = self.next_id;
        self.next_id = self.next_id.checked_add(1).ok_or_else(|| {
            CodexSubscriptionError::Protocol("request id space exhausted".into())
        })?;

        self.write_message(&json!({
            "id": id,
            "method": method,
            "params": params,
        }))?;
        self.read_response(id)
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<(), CodexSubscriptionError> {
        self.write_message(&json!({
            "method": method,
            "params": params,
        }))
    }
}

pub struct StdioCodexTransport {
    inner: JsonlCodexTransport<BufReader<ChildStdout>, ChildStdin>,
    child: Child,
}

impl StdioCodexTransport {
    pub fn spawn(binary: impl AsRef<OsStr>) -> Result<Self, CodexSubscriptionError> {
        let mut child = Command::new(binary)
            .arg("app-server")
            .arg("--stdio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|error| CodexSubscriptionError::ProviderUnavailable(error.to_string()))?;

        let stdin = child.stdin.take().ok_or_else(|| {
            CodexSubscriptionError::ProviderUnavailable(
                "codex app-server stdin was not available".into(),
            )
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            CodexSubscriptionError::ProviderUnavailable(
                "codex app-server stdout was not available".into(),
            )
        })?;

        Ok(Self {
            inner: JsonlCodexTransport::new(BufReader::new(stdout), stdin),
            child,
        })
    }

    pub fn pop_notification(&mut self) -> Option<Value> {
        self.inner.pop_notification()
    }
}

impl CodexRpcTransport for StdioCodexTransport {
    fn request(&mut self, method: &str, params: Value) -> Result<Value, CodexSubscriptionError> {
        self.inner.request(method, params)
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<(), CodexSubscriptionError> {
        self.inner.notify(method, params)
    }
}

impl std::fmt::Debug for StdioCodexTransport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StdioCodexTransport")
            .field("child_id", &self.child.id())
            .finish_non_exhaustive()
    }
}

impl Drop for StdioCodexTransport {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
