use std::{
    io::{Cursor, Write},
    sync::{Arc, Mutex},
};

use forge_core::{
    CodexRpcTransport, CodexSubscriptionError, JsonlCodexTransport, StdioCodexTransport,
};
use serde_json::json;

#[derive(Clone, Default)]
struct SharedWriter(Arc<Mutex<Vec<u8>>>);

impl SharedWriter {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().expect("writer lock").clone()).expect("utf8")
    }
}

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("writer lock").extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn jsonl_transport_matches_response_ids_and_queues_notifications() {
    let input = concat!(
        "{\"method\":\"account/updated\",\"params\":{\"authMode\":\"chatgpt\",\"planType\":\"plus\"}}\n",
        "{\"id\":0,\"result\":{\"account\":{\"type\":\"chatgpt\",\"planType\":\"plus\"}}}\n",
        "{\"id\":1,\"result\":{\"rateLimits\":{\"primary\":null}}}\n"
    );
    let writer = SharedWriter::default();
    let writer_view = writer.clone();
    let mut transport = JsonlCodexTransport::new(Cursor::new(input.as_bytes().to_vec()), writer);

    let account = transport
        .request("account/read", json!({}))
        .expect("first response");
    let limits = transport
        .request("account/rateLimits/read", json!({}))
        .expect("second response");

    assert_eq!(account["account"]["planType"], "plus");
    assert!(limits.get("rateLimits").is_some());
    assert_eq!(
        writer_view.text(),
        concat!(
            "{\"id\":0,\"method\":\"account/read\",\"params\":{}}\n",
            "{\"id\":1,\"method\":\"account/rateLimits/read\",\"params\":{}}\n"
        )
    );

    let notification = transport
        .pop_notification()
        .expect("notification was queued");
    assert_eq!(notification["method"], "account/updated");
    assert_eq!(notification["params"]["planType"], "plus");
}

#[test]
fn jsonl_transport_writes_notifications_without_request_ids() {
    let writer = SharedWriter::default();
    let writer_view = writer.clone();
    let mut transport = JsonlCodexTransport::new(Cursor::new(Vec::<u8>::new()), writer);

    transport
        .notify("initialized", json!({}))
        .expect("notification writes");

    assert_eq!(
        writer_view.text(),
        "{\"method\":\"initialized\",\"params\":{}}\n"
    );
}

#[test]
fn malformed_json_and_rpc_errors_fail_closed() {
    let malformed_writer = SharedWriter::default();
    let mut malformed =
        JsonlCodexTransport::new(Cursor::new(b"not-json\n".to_vec()), malformed_writer);
    let malformed_error = malformed
        .request("account/read", json!({}))
        .expect_err("malformed response must fail");
    assert!(matches!(
        malformed_error,
        CodexSubscriptionError::Protocol(_)
    ));

    let error_writer = SharedWriter::default();
    let mut rpc_error = JsonlCodexTransport::new(
        Cursor::new(
            b"{\"id\":0,\"error\":{\"code\":-32001,\"message\":\"Server overloaded; retry later.\"}}\n"
                .to_vec(),
        ),
        error_writer,
    );
    let error = rpc_error
        .request("account/read", json!({}))
        .expect_err("RPC error must fail");
    assert!(matches!(error, CodexSubscriptionError::Protocol(_)));
}

#[test]
fn missing_codex_binary_is_provider_unavailable() {
    let error = StdioCodexTransport::spawn("/definitely/missing/autodev-codex")
        .expect_err("missing binary must fail clearly");

    assert!(matches!(
        error,
        CodexSubscriptionError::ProviderUnavailable(_)
    ));
}
