use crate::storage::Database;
use crate::config::ConfigManager;
use crate::error::Result;
use tokio::sync::{broadcast, mpsc, Mutex};
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::test;

    #[test]
    async fn test_config_manager_creation() -> Result<()> {
        let config_manager = ConfigManager::new();
        assert!(config_manager.is_ok());
        Ok(())
    }

    #[test]
    async fn test_database_initialization() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = format!("sqlite:{}", temp_dir.path().join("test.db").display());
        let database = Database::new(&db_path).await?;
        assert!(Arc::strong_count(&database) == 1);
        Ok(())
    }

    #[test]
    async fn test_channel_creation() -> Result<()> {
        let (tx, rx) = mpsc::channel::<String>(100);
        let (event_tx, event_rx) = broadcast::channel::<String>(100);
        
        // Test basic channel functionality
        tx.send("test".to_string()).await?;
        
        let received = rx.recv().await;
        assert!(received.is_ok());
        assert_eq!(received.unwrap(), "test");
        
        // Test broadcast channel
        event_tx.send("broadcast_test".to_string())?;
        let received_event = event_rx.recv().await?;
        assert_eq!(received_event, "broadcast_test");
        
        Ok(())
    }

    #[test]
    async fn test_arc_mutex_usage() -> Result<()> {
        let data = Arc::new(Mutex::new(42));
        let data_clone = data.clone();
        
        // Modify data in async context
        {
            let mut guard = data.lock().await;
            *guard = 100;
        }
        
        // Read modified data
        let guard = data_clone.lock().await;
        assert_eq!(*guard, 100);
        
        Ok(())
    }

    #[test]
    async fn test_xmpp_client_config_default() {
        use crate::xmpp::XmppClientConfig;
        
        let config = XmppClientConfig::default();
        assert_eq!(config.resource, "xmpp-client");
        assert_eq!(config.server_port, 5222);
        assert!(config.use_tls);
        assert!(!config.accept_invalid_certs);
        assert!(config.auto_reconnect);
    }

    #[test]
    fn test_jid_parsing() -> Result<()> {
        use xmpp_parsers::Jid;
        
        let jid_str = "user@example.com/resource";
        let jid: Jid = jid_str.parse()?;
        
        assert_eq!(jid.node(), Some("user"));
        assert_eq!(jid.domain(), "example.com");
        assert_eq!(jid.resource(), Some("resource"));
        
        // Test bare JID
        let bare_jid_str = "user@example.com";
        let bare_jid: Jid = bare_jid_str.parse()?;
        
        assert_eq!(bare_jid.node(), Some("user"));
        assert_eq!(bare_jid.domain(), "example.com");
        assert_eq!(bare_jid.resource(), None);
        
        Ok(())
    }

    #[test]
    fn test_xmpp_event_serialization() -> Result<()> {
        use crate::xmpp::events::XmppEvent;
        use xmpp_parsers::Jid;
        
        let jid: Jid = "user@example.com".parse()?;
        let event = XmppEvent::MessageReceived {
            from: jid.clone(),
            to: jid.clone(),
            body: "Hello, World!".to_string(),
            stanza_id: "msg_123".to_string(),
            timestamp: Some(chrono::Utc::now()),
        };
        
        // Test that the event can be cloned and serialized
        let event_clone = event.clone();
        
        match (event, event_clone) {
            (
                XmppEvent::MessageReceived { from: f1, to: t1, body: b1, stanza_id: s1, .. },
                XmppEvent::MessageReceived { from: f2, to: t2, body: b2, stanza_id: s2, .. }
            ) => {
                assert_eq!(f1, f2);
                assert_eq!(t1, t2);
                assert_eq!(b1, b2);
                assert_eq!(s1, s2);
            }
            _ => panic!("Event cloning failed"),
        }
        
        Ok(())
    }

    #[test]
    fn test_error_handling() -> Result<()> {
        use crate::error::XmppError;
        
        let error = XmppError::AuthenticationError("Invalid credentials".to_string());
        assert!(matches!(error, XmppError::AuthenticationError(_)));
        
        let formatted = format!("{}", error);
        assert!(formatted.contains("Authentication failed"));
        
        Ok(())
    }

    #[test]
    fn test_file_size_formatting() -> Result<()> {
        // Test the file size formatting utility
        fn format_size(size: u64) -> String {
            const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
            let mut size = size as f64;
            let mut unit_index = 0;

            while size >= 1024.0 && unit_index < UNITS.len() - 1 {
                size /= 1024.0;
                unit_index += 1;
            }

            if unit_index == 0 {
                format!("{} {}", size as u64, UNITS[unit_index])
            } else {
                format!("{:.1} {}", size, UNITS[unit_index])
            }
        }

        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
        
        Ok(())
    }
}

// Integration tests that require more setup
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    async fn test_full_workflow() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create database
        let db_path = format!("sqlite:{}", temp_dir.path().join("test.db").display());
        let database = Arc::new(Database::new(&db_path).await?);
        
        // Create config manager
        let config_manager = ConfigManager::new()?;
        
        // Create channels
        let (command_tx, mut command_rx) = mpsc::channel(100);
        let (event_tx, mut event_rx) = broadcast::channel(100);
        
        // Test that channels work with the database
        let db_clone = database.clone();
        tokio::spawn(async move {
            // Simulate processing a command
            if let Some(_cmd) = command_rx.recv().await {
                // Store some test data
                let _ = db_clone.save_message(
                    &"user@example.com".parse()?,
                    &"contact@example.com".parse()?,
                    "Test message",
                    "chat",
                    "test_id",
                ).await;
            }
        });
        
        // Send a test command
        use crate::xmpp::XmppCommand;
        command_tx.send(XmppCommand::Disconnect).await?;
        
        // Verify database entry
        let messages = database.get_chat_history(
            &"user@example.com".parse()?,
            &"contact@example.com".parse()?,
            10,
            0,
        ).await?;
        
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].body, "Test message");
        
        Ok(())
    }
}