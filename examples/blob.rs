use std::{fs, io, path::PathBuf};

use static_load::{Resource, ResourceCell};

struct FileData(Vec<u8>);

impl Resource for FileData {
    type Defintion = PathBuf;
    type LoadError = io::Error;

    async fn load(path: &Self::Defintion) -> Result<Self, Self::LoadError> {
        // you can add validation, deserialization, etc here
        Ok(Self(fs::read(path)?))
    }
}

const BLOB: ResourceCell<FileData> = ResourceCell::define();

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // You will probably get the definition from cli args or config
    let path = "./large_file".into();

    BLOB.init(path).await.unwrap();

    any_function().await
}

async fn any_function() {
    // Reading a resource is very fast (and doesn't require .await!)
    let blob = BLOB.read();
    println!("The length of blob is: {} bytes", blob.0.len());

    // You can trigger a reload from anywhere
    BLOB.reload().await.unwrap();
}
