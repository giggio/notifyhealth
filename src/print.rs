use bollard::models::HealthStatusEnum;
use itertools::Itertools;

use crate::containers::{ContainerStatus, RunningContainerStatus};

pub fn running_containers(running_containers: Vec<RunningContainerStatus>) {
    if running_containers.is_empty() {
        println!("No running containers.");
    } else {
        let containers_grouped_by_health = running_containers.into_iter().group_by(|c| c.health);
        for (health_status, group) in containers_grouped_by_health.into_iter() {
            match health_status {
                Some(HealthStatusEnum::UNHEALTHY) => {
                    println!("Running, unhealthy containers:");
                    for container in group {
                        println!("{name}", name = container.name);
                    }
                }
                Some(status) => {
                    println!("Running containers ({status}):");
                    for container in group {
                        println!("{name}", name = container.name);
                    }
                }
                None => {
                    println!("Running containers without health status:");
                    for container in group {
                        println!("{name}", name = container.name);
                    }
                }
            }
        }
    }
}

pub fn stopped_containers(stopped_containers: Vec<ContainerStatus>) {
    if stopped_containers.is_empty() {
        println!("No container that was supposed to be running is stopped.");
    } else {
        println!("The following containers are stopped:");
        for container in stopped_containers.into_iter() {
            println!("{name}", name = container.name);
        }
    }
}
