use reqwest::{self, Error};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct Context {
    pub label: String,
    pub value: String,
}
impl Context {
    /// Convert into formatted string
    fn formatted(&self) -> String {
        format!(">`{}`: {}\n", self.label, self.value)
    }
}

#[derive(Deserialize)]
pub struct Notification {
    pub message: String,
    pub timestamp: String,
    pub context: Vec<Context>,
}
impl Notification {
    /// Consume the `Notification` and send it to a given destination (API endpoint)
    pub async fn send(self, destination: &str) -> Result<(), Error> {
        // Initiate the HTTP client
        let http_client = reqwest::Client::new();

        // Parse the `Notification` into a slack message
        let slack_message = self.into_slack_message();

        // Build and send the HTTP request to a given destination
        // with the payload being our derived slack message
        http_client
            .post(destination)
            .header("Content-type", "application/json")
            .body(slack_message)
            .send()
            .await?;

        Ok(())
    }

    /// Consume the `Notification` and parse it into a message (String)
    fn into_message(self) -> String {
        let mut message = format!(
            "`Issue`: {}\n>`Timestamp`: _{}_\n",
            self.message, self.timestamp
        );
        for ctx in self.context {
            message.push_str(&ctx.formatted());
        }

        message
    }

    /// Consume the `Notification` and parse it into a slack message (JSON String)
    fn into_slack_message(self) -> String {
        let message = self.into_message();

        // Build the JSON payload required for a slack message
        json!({
            "blocks": vec![
                json!({
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": message,
                    }
                })
            ]
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Context, Notification};

    /// Test Case Structure
    struct TestCase {
        expected_message: String,
        expected_slack_message: String,
        expected_context: Vec<String>,
        notification: Notification,
    }

    /// A test to make sure context parsing is working
    #[test]
    fn can_parse_context() {
        let scenarios = get_scenarios();
        for scenario in scenarios {
            for (idx, ctx) in scenario.notification.context.iter().enumerate() {
                let actual = ctx.formatted();
                let expected = scenario.expected_context[idx].clone();
                assert_eq!(actual, expected)
            }
        }
    }

    /// A test to make sure message parsing is working
    #[test]
    fn can_parse_into_message() {
        let scenarios = get_scenarios();
        for scenario in scenarios {
            let actual = scenario.notification.into_message();
            let expected = scenario.expected_message;
            assert_eq!(actual, expected)
        }
    }

    /// A test to make sure slack message parsing is working
    #[test]
    fn can_parse_into_slack_message() {
        let scenarios = get_scenarios();
        for scenario in scenarios {
            let actual = scenario.notification.into_slack_message();
            let expected = scenario.expected_slack_message;
            assert_eq!(actual, expected)
        }
    }

    /// Test case scenarios for each test to use
    fn get_scenarios() -> Vec<TestCase> {
        vec![
            TestCase {
                expected_context: vec![String::from(">`Customer ID`: 0\n")],
                expected_message: String::from("`Issue`: External API Error: Could not find API Keys\n>`Timestamp`: _2024-01-19 19:26:20.022233_\n>`Customer ID`: 0\n"),
                expected_slack_message: String::from("{\"blocks\":[{\"text\":{\"text\":\"`Issue`: External API Error: Could not find API Keys\\n>`Timestamp`: _2024-01-19 19:26:20.022233_\\n>`Customer ID`: 0\\n\",\"type\":\"mrkdwn\"},\"type\":\"section\"}]}"),
                notification: Notification {
                    message: String::from("External API Error: Could not find API Keys"),
                    timestamp: String::from("2024-01-19 19:26:20.022233"),
                    context: vec![Context {
                        label: String::from("Customer ID"),
                        value: String::from("0"),
                    }],
                },
            },
            TestCase {
                expected_context: vec![String::from(">`Customer ID`: 0\n"), String::from(">`Transaction ID`: 0d738c014b6a00ddb68edafc\n")],
                expected_message: String::from("`Issue`: Payment Proccessing Error: Failed to capture transaction\n>`Timestamp`: _2024-01-18 21:06:05.778504_\n>`Customer ID`: 0\n>`Transaction ID`: 0d738c014b6a00ddb68edafc\n"),
                expected_slack_message: String::from("{\"blocks\":[{\"text\":{\"text\":\"`Issue`: Payment Proccessing Error: Failed to capture transaction\\n>`Timestamp`: _2024-01-18 21:06:05.778504_\\n>`Customer ID`: 0\\n>`Transaction ID`: 0d738c014b6a00ddb68edafc\\n\",\"type\":\"mrkdwn\"},\"type\":\"section\"}]}"),
                notification: Notification {
                    message: String::from("Payment Proccessing Error: Failed to capture transaction"),
                    timestamp: String::from("2024-01-18 21:06:05.778504"),
                    context: vec![
                        Context {
                            label: String::from("Customer ID"),
                            value: String::from("0"),
                        },
                        Context {
                            label: String::from("Transaction ID"),
                            value: String::from("0d738c014b6a00ddb68edafc"),
                        }
                    ],
                }
            },
            TestCase {
                expected_context: vec![String::from(">`Customer ID`: 0\n"), String::from(">`Payment Link`: 7ea9ab4001d87d81207be05\n")],
                expected_message: String::from("`Issue`: Payment Link Error: Missing Order ID for level 3 data\n>`Timestamp`: _2024-01-18 16:41:04.563205_\n>`Customer ID`: 0\n>`Payment Link`: 7ea9ab4001d87d81207be05\n"),
                expected_slack_message: String::from("{\"blocks\":[{\"text\":{\"text\":\"`Issue`: Payment Link Error: Missing Order ID for level 3 data\\n>`Timestamp`: _2024-01-18 16:41:04.563205_\\n>`Customer ID`: 0\\n>`Payment Link`: 7ea9ab4001d87d81207be05\\n\",\"type\":\"mrkdwn\"},\"type\":\"section\"}]}"),
                notification: Notification {
                    message: String::from("Payment Link Error: Missing Order ID for level 3 data"),
                    timestamp: String::from("2024-01-18 16:41:04.563205"),
                    context: vec![
                        Context {
                            label: String::from("Customer ID"),
                            value: String::from("0"),
                        },
                        Context {
                            label: String::from("Payment Link"),
                            value: String::from("7ea9ab4001d87d81207be05"),
                        }
                    ],
                },
            }
        ]
    }
}
