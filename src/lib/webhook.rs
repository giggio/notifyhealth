use super::containers::{RunningContainerStatus, StoppedContainerStatus};
use isahc::{Body, Error, HttpClient, Request, Response};
use log::*;
#[cfg(test)]
use mockall::automock;
use serde::{Deserialize, Serialize};
use std::io::Read;

pub struct MyHttpClient {
    pub client: HttpClient,
}

#[cfg_attr(test, automock)]
trait SendsHttp {
    fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Body>, Error>;
}
impl SendsHttp for MyHttpClient {
    fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Body>, Error> {
        self.client.send(request)
    }
}

pub type FormatMessageType = fn(
    running_containers: &[RunningContainerStatus],
    stopped_containers: &[StoppedContainerStatus],
    hostname: Option<String>,
) -> Result<Vec<u8>, serde_json::Error>;

pub struct Webhook {
    http_client: Box<dyn SendsHttp + Sync>,
    message_formatter: Option<FormatMessageType>,
}

impl Default for Webhook {
    fn default() -> Self {
        Webhook {
            http_client: Box::new(MyHttpClient {
                client: HttpClient::new().expect("shared client failed to initialize"),
            }),
            message_formatter: None,
        }
    }
}

impl Webhook {
    pub fn new(message_formatter: Option<FormatMessageType>) -> Self {
        Webhook {
            http_client: Box::new(MyHttpClient {
                client: HttpClient::new().expect("shared client failed to initialize"),
            }),
            message_formatter,
        }
    }

    pub fn notify(
        &self,
        url: &str,
        running_containers: Vec<RunningContainerStatus>,
        stopped_containers: Vec<StoppedContainerStatus>,
        hostname: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if (running_containers.len() + stopped_containers.len()) == 0 {
            return Ok(());
        }
        let body_bytes = if let Some(message_formatter) = &self.message_formatter {
            message_formatter(&running_containers, &stopped_containers, hostname)?
        } else {
            serde_json::to_vec(
                &(WebHookNotifyBody {
                    running_containers,
                    stopped_containers,
                    hostname,
                }),
            )?
        };
        let req = Request::post(url)
            .header("content-type", "application/json")
            .body(body_bytes)?;
        let mut res = self.http_client.send(req)?;
        let mut body = String::new();
        res.body_mut().read_to_string(&mut body)?;
        if !res.status().is_success() {
            return Err(format!("Error: status code: {status}. Body: {body}", status = res.status()).into());
        } else {
            info!("Response: status code: {status}. Body: {body}", status = res.status());
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct WebHookNotifyBody {
    pub running_containers: Vec<RunningContainerStatus>,
    pub stopped_containers: Vec<StoppedContainerStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::HealthStatusEnum;
    use isahc::{Body, Response};

    #[tokio::test]
    async fn check_webhook_notify() {
        let mut client = MockSendsHttp::new();
        const URL: &str = "http://localhost:8080/";
        let running_containers = vec![RunningContainerStatus {
            name: "test1".to_string(),
            health: None,
        }];
        let stopped_containers = vec![StoppedContainerStatus {
            name: "test2".to_string(),
            status: Some("exited".to_string()),
        }];
        let rc = running_containers.clone();
        let sc = stopped_containers.clone();
        client
            .expect_send()
            .withf(move |req| {
                *req.uri() == URL
                    && req.method() == "POST"
                    && serde_json::from_slice::<WebHookNotifyBody>(req.body()).unwrap()
                        == WebHookNotifyBody {
                            running_containers: rc.clone(),
                            stopped_containers: sc.clone(),
                            hostname: None,
                        }
            })
            .times(1)
            .return_once(|_| Ok(Response::builder().status(200).body(Body::from("")).unwrap()));
        let webhook = Webhook {
            http_client: Box::new(client),
            message_formatter: None,
        };
        webhook
            .notify(URL, running_containers, stopped_containers, None)
            .unwrap();
    }

    #[tokio::test]
    async fn check_webhook_notify_with_host() {
        let mut client = MockSendsHttp::new();
        const URL: &str = "http://localhost:8080/";
        let running_containers = vec![
            RunningContainerStatus {
                name: "test1".to_string(),
                health: None,
            },
            RunningContainerStatus {
                name: "test3".to_string(),
                health: Some(HealthStatusEnum::UNHEALTHY),
            },
        ];
        let stopped_containers = vec![StoppedContainerStatus {
            name: "test2".to_string(),
            status: Some("exited".to_string()),
        }];
        let rc = running_containers.clone();
        let sc = stopped_containers.clone();
        client
            .expect_send()
            .withf(move |req| {
                *req.uri() == URL
                    && req.method() == "POST"
                    && serde_json::from_slice::<WebHookNotifyBody>(req.body()).unwrap()
                        == WebHookNotifyBody {
                            running_containers: rc.clone(),
                            stopped_containers: sc.clone(),
                            hostname: Some("myhostname".to_owned()),
                        }
            })
            .times(1)
            .return_once(|_| Ok(Response::builder().status(200).body(Body::from("")).unwrap()));
        let webhook = Webhook {
            http_client: Box::new(client),
            message_formatter: None,
        };
        webhook
            .notify(
                URL,
                running_containers,
                stopped_containers,
                Some("myhostname".to_owned()),
            )
            .unwrap();
    }

    #[tokio::test]
    async fn when_there_is_no_problem_do_not_notify() {
        let client = MockSendsHttp::new();
        const URL: &str = "http://localhost:8080/";
        let running_containers = vec![];
        let stopped_containers = vec![];
        let webhook = Webhook {
            http_client: Box::new(client),
            message_formatter: None,
        };
        webhook
            .notify(URL, running_containers, stopped_containers, None)
            .unwrap();
    }
}
