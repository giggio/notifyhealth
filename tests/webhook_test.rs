use mockito::Matcher;
use notifyhealth::{
    containers::{RunningContainerStatus, StoppedContainerStatus},
    webhook::Webhook,
};
use serde_json::json;
#[tokio::test]
async fn check_webhook_notify() {
    let webhook = Webhook::default();
    let running_containers = vec![RunningContainerStatus {
        name: "test1".to_string(),
        health: None,
    }];
    let stopped_containers = vec![StoppedContainerStatus {
        name: "test2".to_string(),
        status: Some("exited".to_string()),
    }];
    let url = &mockito::server_url();
    let mock = mockito::mock("POST", "/")
        .match_body(Matcher::Json(json!({"running_containers": [{"name": "test1"}], "stopped_containers": [{"name": "test2", "status": "exited"}]})))
        .match_header("content-type", "application/json")
        .with_status(201)
        .create();
    webhook
        .notify(url, running_containers, stopped_containers, None)
        .unwrap();
    mock.assert();
}
