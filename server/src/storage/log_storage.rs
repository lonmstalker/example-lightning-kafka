use crate::data::storage_data::Message;
use std::io::{Result as IoResult};

pub trait LogStorage {

    /// Добавляет (append) новое сообщение в лог партиции.
    /// Возвращает offset (смещение), по которому сообщение записано.
    async fn append(&mut self, partition_id: &str, msg: &Message) -> IoResult<u64>;

    /// Считывает сообщения, начиная с offset.
    /// Возвращает вектор (не более limit сообщений), либо пустой, если достигли конца.
    async fn read(&self, partition_id: &str, offset: u64, limit: usize) -> IoResult<Vec<(u64, Message)>>;
}