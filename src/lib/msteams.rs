use super::containers::{RunningContainerStatus, StoppedContainerStatus};
use log::*;
use mhteams::{Fact, Message, Section};

pub fn format_message(
    running_containers: &[RunningContainerStatus],
    stopped_containers: &[StoppedContainerStatus],
) -> Result<Vec<u8>, serde_json::Error> {
    let mut sections = vec![];
    if !running_containers.is_empty() {
        sections.push(
            Section::new().text("The following containers are unhealthy:").facts(
                running_containers
                    .iter()
                    .map(|c| Fact::new("Name", c.name.clone()))
                    .collect(),
            ),
        );
        warn!("Sections after unhealthy: {:?}", sections);
    }
    if !stopped_containers.is_empty() {
        sections.push(
            Section::new().text("The following containers are stopped:").facts(
                stopped_containers
                    .iter()
                    .map(|c| Fact::new("Name", c.name.clone()))
                    .collect(),
            ),
        );
        warn!("Sections after stopped: {:?}", sections);
    }
    let msg = Message::new() // todo add server name
        .title("Problem in containers! 🤕")
        .summary("Problems in containers")
        .sections(sections);
    info!("Message to be sent: {:?}", msg);
    serde_json::to_vec(&msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn check_message() {
        let running_containers = vec![RunningContainerStatus {
            name: "test1".to_string(),
            health: None,
        }];
        let stopped_containers = vec![StoppedContainerStatus {
            name: "test2".to_string(),
            status: Some("exited".to_string()),
        }];
        let formatted_message_bytes = format_message(&running_containers, &stopped_containers).unwrap();

        let msg = Message::new()
            .title("Problem in containers! 🤕")
            .summary("Problems in containers")
            .sections(vec![
                Section::new()
                    .text("The following containers are unhealthy:")
                    .facts(vec![Fact::new("Name", "test1")]),
                Section::new()
                    .text("The following containers are stopped:")
                    .facts(vec![Fact::new("Name", "test2")]),
            ]);
        let expected_message_bytes = serde_json::to_vec::<Message>(&msg).unwrap();
        let expected_message = std::str::from_utf8(&expected_message_bytes).unwrap();
        let formatted_message = std::str::from_utf8(&formatted_message_bytes).unwrap();
        assert_eq!(formatted_message, expected_message);
    }
}
