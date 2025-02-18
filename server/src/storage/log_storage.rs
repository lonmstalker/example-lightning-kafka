use crate::data::exception::FileStorageError;
use crate::data::file_storage_data::FileData;

pub trait FileWriter {
    async fn write(topic: &String, data: FileData) -> Result<bool, FileStorageError>;
}

pub trait FileReader {
    async fn read(topic: String) -> Result<FileData, FileStorageError>;
}