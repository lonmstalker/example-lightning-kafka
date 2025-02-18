use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub file_id: Option<String>, // Идентификатор файла (если это чанк большого файла)
    pub chunk_index: Option<String>, // Какой по счёту чанк
    pub total_chunks: Option<String>, // Общее кол-во чанков, если это фрагмент одного файла

    pub payload: Vec<u8>,
}