use prost::Message;
use serde_json::Value;
use std::{collections::HashMap, error::Error};

pub trait MessageFactory: Send + Sync {
    fn decode(&self, bytes: &[u8]) -> Result<String, Box<dyn Error + Send + Sync>>;
    fn encode(&self, json: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>>;
    fn get_schema(&self) -> String;
}

struct ProtoMessageFactory<T: Message + Default + serde::Serialize> {
    phantom: std::marker::PhantomData<T>,
}

impl<T: Message + Default + serde::Serialize + serde::de::DeserializeOwned> ProtoMessageFactory<T> {
    fn new() -> Self {
        Self {
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Message + Default + serde::Serialize + serde::de::DeserializeOwned> MessageFactory
    for ProtoMessageFactory<T>
{
    fn decode(&self, bytes: &[u8]) -> Result<String, Box<dyn Error + Send + Sync>> {
        let msg = T::decode(bytes)?;
        Ok(serde_json::to_string_pretty(&msg)?)
    }

    fn encode(&self, json: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let value: Value = serde_json::from_str(json)?;
        let msg: T = serde_json::from_value(value)?;
        let mut buf = Vec::new();
        msg.encode(&mut buf)?;
        Ok(buf)
    }

    fn get_schema(&self) -> String {
        // For now, return a placeholder. In a full implementation,
        // we could use the protobuf descriptors to generate this.
        "Schema not available yet".to_string()
    }
}

pub struct MessageRegistry {
    factories: HashMap<String, Box<dyn MessageFactory>>,
}

impl MessageRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    pub fn register<T>(&mut self, name: &str)
    where
        T: Message + Default + serde::Serialize + serde::de::DeserializeOwned + 'static,
    {
        self.factories
            .insert(name.to_string(), Box::new(ProtoMessageFactory::<T>::new()));
    }

    pub fn decode(
        &self,
        msg_type: &str,
        bytes: &[u8],
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.factories
            .get(msg_type)
            .ok_or_else(|| format!("Unknown message type: {}", msg_type).into())
            .and_then(|factory| factory.decode(bytes))
    }

    pub fn encode(
        &self,
        msg_type: &str,
        json: &str,
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        self.factories
            .get(msg_type)
            .ok_or_else(|| format!("Unknown message type: {}", msg_type).into())
            .and_then(|factory| factory.encode(json))
    }

    pub fn list_types(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }

    pub fn get_schema(&self, msg_type: &str) -> Option<String> {
        self.factories.get(msg_type).map(|f| f.get_schema())
    }
}
