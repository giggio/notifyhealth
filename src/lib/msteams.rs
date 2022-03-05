use super::containers::{RunningContainerStatus, StoppedContainerStatus};
use itertools::Itertools;
use log::*;
use mhteams::{Fact, Message, Section};

pub fn format_message(
    running_containers: &[RunningContainerStatus],
    stopped_containers: &[StoppedContainerStatus],
    hostname: Option<String>,
) -> Result<Vec<u8>, serde_json::Error> {
    let mut sections = vec![];
    if !running_containers.is_empty() {
        for (health_opt, group) in &running_containers.iter().group_by(|c| &c.health) {
            if let Some(health) = health_opt {
                sections.push(
                    Section::new()
                        .text("The following running containers are not healthy:")
                        .facts(group.into_iter().map(|c| Fact::new(c.name.clone(), health)).collect()),
                );
            } else {
                sections.push(
                    Section::new()
                        .text("The following running containers have no health status:")
                        .facts(
                            group
                                .into_iter()
                                .map(|c| Fact::new(c.name.clone(), "no health status"))
                                .collect(),
                        ),
                );
            }
        }
        warn!("Sections after unhealthy: {:?}", sections);
    }
    if !stopped_containers.is_empty() {
        sections.push(
            Section::new().text("The following containers are not running:").facts(
                stopped_containers
                    .iter()
                    .map(|c| {
                        if let Some(status) = &c.status {
                            Fact::new(c.name.clone(), status.clone())
                        } else {
                            Fact::new(c.name.clone(), "no status")
                        }
                    })
                    .collect(),
            ),
        );
        warn!("Sections after stopped: {:?}", sections);
    }
    let mut msg = Message::new() // todo add server name
        .title("Problem in containers! 🤕")
        .summary("Problems in containers");
    if let Some(hostname) = hostname {
        msg = msg.text(format!("Server: `{hostname}`."));
    }
    msg = msg.sections(sections);
    info!("Message to be sent: {:?}", msg);
    serde_json::to_vec(&msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::HealthStatusEnum;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn check_message() {
        let running_containers = vec![
            RunningContainerStatus {
                name: "test1".to_string(),
                health: None,
            },
            RunningContainerStatus {
                name: "test2".to_string(),
                health: Some(HealthStatusEnum::UNHEALTHY),
            },
        ];
        let stopped_containers = vec![
            StoppedContainerStatus {
                name: "test3".to_string(),
                status: Some("exited".to_string()),
            },
            StoppedContainerStatus {
                name: "test4".to_string(),
                status: None,
            },
        ];
        let formatted_message_bytes =
            format_message(&running_containers, &stopped_containers, Some("myhostname".to_owned())).unwrap();

        let msg = Message::new()
            .title("Problem in containers! 🤕")
            .summary("Problems in containers")
            .text("Server: `myhostname`.")
            .sections(vec![
                Section::new()
                    .text("The following running containers have no health status:")
                    .facts(vec![Fact::new("test1", "no health status")]),
                Section::new()
                    .text("The following running containers are not healthy:")
                    .facts(vec![Fact::new("test2", "unhealthy")]),
                Section::new()
                    .text("The following containers are not running:")
                    .facts(vec![Fact::new("test3", "exited"), Fact::new("test4", "no status")]),
            ]);
        let expected_message_bytes = serde_json::to_vec::<Message>(&msg).unwrap();
        let expected_message = std::str::from_utf8(&expected_message_bytes).unwrap();
        let formatted_message = std::str::from_utf8(&formatted_message_bytes).unwrap();
        assert_eq!(formatted_message, expected_message);
    }
}
