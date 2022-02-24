use bollard::container::{InspectContainerOptions, ListContainersOptions};
use bollard::models::{ContainerStateStatusEnum, HealthStatusEnum};
use bollard::Docker;
use std::collections::HashMap;

pub async fn check_running_containers(docker: &Docker) -> Result<(), Box<dyn std::error::Error>> {
    let mut filter = HashMap::new();
    filter.insert("status", vec!["running"]);
    filter.insert("health", vec!["unhealthy", "starting", "none"]);
    let containers = &docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: filter,
            ..Default::default()
        }))
        .await?;

    if containers.is_empty() {
        println!("No unhealthy containers.");
        return Ok(());
    }
    for container in containers {
        let names = container.names.as_ref().unwrap();
        let mut name: &str = names.first().unwrap();
        if name.starts_with('/') {
            name = &name[1..];
        }
        println!("Inpecting container {name}.");
        let inpect_result = &docker.inspect_container(name, None::<InspectContainerOptions>).await?;
        if inpect_result.state.is_none() {
            eprintln!("No state for container {name}");
            continue;
        }
        let state = inpect_result.state.as_ref().unwrap();
        match state.status {
            None => {
                eprintln!("No status for container {name}");
                continue;
            }
            Some(status) => {
                if status != ContainerStateStatusEnum::RUNNING {
                    eprintln!("Container {name} is not running, it is {status}");
                    continue;
                }
            }
        }
        match &state.health {
            None => {
                eprintln!("No health for container {name}");
                continue;
            }
            Some(health) => match health.status {
                None => {
                    eprintln!("No status for container {name}");
                    continue;
                }
                Some(HealthStatusEnum::HEALTHY) => {
                    printlnv!("Container {name} is unhealthy");
                }
                Some(HealthStatusEnum::UNHEALTHY) => {
                    eprintln!("Container {name} is unhealthy");
                }
                Some(status) => {
                    eprintln!("Container {name} is not healthy, it is {status}");
                }
            },
        }
    }
    Ok(())
}

pub async fn check_not_running_containers(docker: &Docker, label: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut filter = HashMap::new();
    filter.insert(
        "status",
        vec!["created", "paused", "restarting", "removing", "exited", "dead"],
    );
    filter.insert("label", vec![label]);
    let containers = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: filter,
            ..Default::default()
        }))
        .await?;
    if containers.is_empty() {
        println!("No stopped containers.");
        return Ok(());
    }
    for container in containers {
        let names = container.names.as_ref().unwrap();
        let mut name: &str = names.first().unwrap();
        if name.starts_with('/') {
            name = &name[1..];
        }
        println!("Container {name} was supposed to be running.");
        let inpect_result = &docker.inspect_container(name, None::<InspectContainerOptions>).await?;
        if inpect_result.state.is_none() {
            eprintln!("Could not fetch inspection state for container {name}.");
            continue;
        }
        let state = inpect_result.state.as_ref().unwrap();
        match state.status {
            None => {
                eprintln!("Could not fetch status for {name}.");
                continue;
            }
            Some(status) => {
                if status != ContainerStateStatusEnum::RUNNING {
                    println!("Container {name} is {status}.");
                }
            }
        }
    }
    Ok(())
}
