use std::collections::HashMap;
use std::io::SeekFrom;
use tokio::fs::{File, OpenOptions};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter};
use crate::data::storage_data::Message;
use crate::storage::log_storage::LogStorage;

pub struct FileLogStorage {
    partitions: HashMap<String, PartitionFile>,
}

pub struct PartitionFile {
    file_path: String,
    current_offset: u64,
    writer: BufWriter<File>,
}

impl FileLogStorage {
    pub fn new() -> Self {
        Self { partitions: HashMap::new() }
    }

    pub async fn open_partition(&mut self, partition_id: &str) -> io::Result<()> {
        if self.partitions.contains_key(partition_id) {
            return Ok(());
        }

        let file_path = format!("{}.log", partition_id);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&file_path).await?;

        let current_offset = file.metadata().await?.len();
        let writer = BufWriter::new(file);

        let part_file = PartitionFile {
            file_path,
            current_offset,
            writer,
        };

        self.partitions.insert(partition_id.to_string(), part_file);
        Ok(())
    }
}

impl LogStorage for FileLogStorage {
    async fn append(&mut self, partition_id: &str, msg: &Message) -> io::Result<u64> {
        self.open_partition(partition_id).await?;
        let partition = self.partitions.get_mut(partition_id).unwrap();

        let data = serde_json::to_vec(msg)?;

        let data_len = data.len() as u64;
        partition.writer.write_all(&data_len.to_le_bytes()).await?;
        partition.writer.write_all(&data).await?;
        partition.writer.flush().await?;

        partition.current_offset += 8 + data_len;

        Ok(partition.current_offset)
    }

    async fn read(&self, partition_id: &str, offset: u64, limit: usize) -> io::Result<Vec<(u64, Message)>> {
        let partition = self.partitions.get(partition_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Partition not found"))?;

        let file = File::open(&partition.file_path).await?;
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(offset)).await?;

        let mut result = Vec::with_capacity(limit);

        for _ in 0..limit {
            let mut len_buf = [0u8; 8];

            if reader.read_exact(&mut len_buf).await.is_err() {
                break;
            }

            let data_len = u64::from_le_bytes(len_buf);
            let mut data_buf = vec![0u8; data_len as usize];

            if reader.read_exact(&mut data_buf).await.is_err() {
                break;
            }

            let msg: Message = serde_json::from_reader(&*data_buf)?;
            let cur_offset = reader.stream_position().await? - (data_len + 8);
            result.push((cur_offset, msg));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::data::storage_data::Message;
    use crate::storage::file_log_storage::FileLogStorage;
    use crate::storage::log_storage::LogStorage;

    #[tokio::test]
    async fn success_test() {
        let part_id = "1";
        let msg = Message {
            file_id: None,
            chunk_index: None,
            total_chunks: None,
            payload: Vec::from("hello".as_bytes()),
        };

        let mut storage = FileLogStorage::new();

        let offset = storage.append(part_id, &msg).await.unwrap();
        let data = storage.read(part_id, 0, 1).await.unwrap();

        assert_eq!(msg.payload, data.first().unwrap().1.payload)
    }
}