use std::time::Instant;

use string_builder::Builder;

#[derive(Debug, Clone)]
pub struct ServerState {
    pub current_connections: u32,
    pub total_connections: u32,
    pub processed_commands: u64,
    pub version: String,
    pub start_time: Instant,
}

impl ServerState {
    pub fn new(version: &str) -> Self {
        Self {
            current_connections: 0,
            total_connections: 0,
            processed_commands: 0,
            version: version.to_owned(),
            start_time: Instant::now(),
        }
    }
    pub fn to_string(self: &Self) -> String {
        let mut str_b = Builder::new(128);
        str_b.append("current connections: ");
        str_b.append(self.current_connections.to_string());
        str_b.append("\n");
        str_b.append("total connections: ");
        str_b.append(self.total_connections.to_string());
        str_b.append("\n");
        str_b.append("processed_commands: ");
        str_b.append(self.processed_commands.to_string());
        str_b.append("\n");
        str_b.append("version: ");
        str_b.append(self.version.as_ref() as &str);
        str_b.append("\n");
        str_b.append("uptime: ");
        str_b.append(
            Instant::now()
                .duration_since(self.start_time)
                .as_secs()
                .to_string(),
        );
        return str_b.string().unwrap();
    }
}
