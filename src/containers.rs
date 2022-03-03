#![warn(clippy::shadow_unrelated)]
use async_trait::async_trait;
use bollard::container::{InspectContainerOptions, ListContainersOptions};
use bollard::errors::Error;
use bollard::models::{ContainerInspectResponse, ContainerSummaryInner, HealthStatusEnum};
use futures_util::Future;
#[cfg(test)]
use mockall::automock;
use std::pin::Pin;

use bollard::Docker;
use futures::prelude::*;

pub struct Containers {
    docker: Docker,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait HasContainers {
    async fn list_containers<'a>(
        &'a self,
        options: Option<ListContainersOptions<&'a str>>,
    ) -> Result<Vec<ContainerSummaryInner>, Error>;
    async fn inspect_container<'a>(
        &'a self,
        container_name: &'a str,
        options: Option<InspectContainerOptions>,
    ) -> Result<ContainerInspectResponse, Error>;
}

impl Containers {
    pub fn new(docker: Docker) -> Self {
        Self { docker }
    }
}

#[async_trait]
impl HasContainers for Containers {
    async fn list_containers<'a>(
        &'a self,
        options: Option<ListContainersOptions<&'a str>>,
    ) -> Result<Vec<ContainerSummaryInner>, Error> {
        self.docker.list_containers(options).await
    }
    fn inspect_container<'a, 'async_trait>(
        &'a self,
        container_name: &'a str,
        options: Option<InspectContainerOptions>,
    ) -> Pin<Box<dyn Future<Output = Result<ContainerInspectResponse, Error>> + Send + 'async_trait>>
    where
        'a: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(self.docker.inspect_container(container_name, options))
    }
}

pub async fn check_running_containers(
    docker: &dyn HasContainers,
) -> Result<Vec<RunningContainerStatus>, Box<dyn std::error::Error>> {
    let filter = hashmap!["status" => vec!["running"], "health" => vec!["unhealthy", "starting", "none"]];
    let containers = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: filter,
            ..Default::default()
        }))
        .await?;
    Ok(future::join_all(containers.into_iter().map(|container| async move {
        let name = get_container_name(&container);
        let inpect_result = docker.inspect_container(name, None).await;
        let health = inpect_result
            .unwrap_or_default()
            .state
            .unwrap_or_default()
            .health
            .unwrap_or_default();
        RunningContainerStatus {
            name: name.to_string(),
            health: health.status,
        }
    }))
    .await)
}

#[derive(Debug, PartialEq)]
pub struct ContainerStatus {
    pub name: String,
    pub status: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct RunningContainerStatus {
    pub name: String,
    pub health: Option<HealthStatusEnum>,
}

pub async fn check_not_running_containers(
    docker: &dyn HasContainers,
    label: &str,
) -> Result<Vec<ContainerStatus>, Box<dyn std::error::Error>> {
    let filter = hashmap!["status" => vec!["created", "paused", "restarting", "removing", "exited", "dead"], "label" => vec![label]];
    let containers = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: filter,
            ..Default::default()
        }))
        .await?;
    Ok(containers
        .into_iter()
        .map(|container| ContainerStatus {
            name: get_container_name(&container).to_string(),
            status: container.state,
        })
        .collect())
}

fn get_container_name(container: &ContainerSummaryInner) -> &str {
    match &container.names {
        Some(names) => {
            let name = names.first().unwrap();
            if let Some(name_without_prefix) = name.strip_prefix('/') {
                name_without_prefix
            } else {
                name
            }
        }
        None => container.id.as_ref().unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::{ContainerState, Health};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn check_running_containers_test() {
        let mut has_containers_mock = MockHasContainers::new();
        let filter = hashmap!["status" => vec!["running"], "health" => vec!["unhealthy", "starting", "none"]];
        has_containers_mock
            .expect_list_containers()
            .withf(move |options| {
                let opt = options.as_ref().unwrap();
                opt.all && opt.filters == filter
            })
            .times(1)
            .returning(|_| {
                Ok(vec![
                    ContainerSummaryInner {
                        names: Some(vec!["/test_container".to_string()]),
                        ..Default::default()
                    },
                    ContainerSummaryInner {
                        names: Some(vec!["/test_container2".to_string()]),
                        ..Default::default()
                    },
                    ContainerSummaryInner {
                        names: Some(vec!["/test_container3".to_string()]),
                        ..Default::default()
                    },
                ])
            });
        has_containers_mock
            .expect_inspect_container()
            .withf(|name, options| name == "test_container" && options.is_none())
            .times(1)
            .returning(|name, _| {
                Ok(ContainerInspectResponse {
                    name: Some(name.to_string()),
                    state: Some(ContainerState {
                        health: Some(Health {
                            status: Some(HealthStatusEnum::UNHEALTHY),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            });
        has_containers_mock
            .expect_inspect_container()
            .withf(|name, options| name == "test_container2" && options.is_none())
            .times(1)
            .returning(|name, _| {
                Ok(ContainerInspectResponse {
                    name: Some(name.to_string()),
                    state: Some(ContainerState {
                        health: None,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            });
        has_containers_mock
            .expect_inspect_container()
            .withf(|name, options| name == "test_container3" && options.is_none())
            .times(1)
            .returning(|name, _| {
                Ok(ContainerInspectResponse {
                    name: Some(name.to_string()),
                    state: None,
                    ..Default::default()
                })
            });
        let running_containers_result = check_running_containers(&has_containers_mock).await;
        if running_containers_result.is_err() {
            panic!(
                "Errors getting running containers: {:?}",
                running_containers_result.err().unwrap()
            );
        }
        let running_containers = running_containers_result.unwrap();
        assert_eq!(
            running_containers,
            vec![
                RunningContainerStatus {
                    name: "test_container".to_string(),
                    health: Some(HealthStatusEnum::UNHEALTHY)
                },
                RunningContainerStatus {
                    name: "test_container2".to_string(),
                    health: None
                },
                RunningContainerStatus {
                    name: "test_container3".to_string(),
                    health: None
                }
            ]
        );
    }

    #[tokio::test]
    async fn check_not_running_containers_test() {
        let mut has_containers_mock = MockHasContainers::new();
        let label = "test_label";
        let filter = hashmap!["status" => vec!["created", "paused", "restarting", "removing", "exited", "dead"], "label" => vec![label]];
        has_containers_mock
            .expect_list_containers()
            .withf(move |options| {
                let opt = options.as_ref().unwrap();
                opt.all && opt.filters == filter
            })
            .times(1)
            .returning(|_| {
                Ok(vec![ContainerSummaryInner {
                    names: Some(vec!["/test_container".to_string()]),
                    state: Some("stopped".to_owned()),
                    ..Default::default()
                }])
            });
        let stopped_containers_result = check_not_running_containers(&has_containers_mock, label).await;
        if stopped_containers_result.is_err() {
            panic!(
                "Errors getting stopped containers: {:?}",
                stopped_containers_result.err().unwrap()
            );
        }
        let stopped_containers = stopped_containers_result.unwrap();
        assert_eq!(
            stopped_containers,
            vec![ContainerStatus {
                name: "test_container".to_string(),
                status: Some("stopped".to_string())
            }]
        );
    }
}
