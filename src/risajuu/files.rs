use reqwest::multipart;
use std::fs::File;
use std::io::Read;

pub async fn upload_file(api_key: &str, file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://generativelanguage.googleapis.com/v1beta/files?key=".to_string() + api_key;

    let mut file = File::open(file_path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let part = multipart::Part::bytes(buf).file_name("upload.bin").mime_str("application/octet-stream")?;

    let form = multipart::Form::new().part("file", part);

    let client = reqwest::Client::new();
    let res = client.post(&url).multipart(form).send().await?.json::<serde_json::Value>().await?;

    // ファイルIDを返す
    Ok(res["name"].as_str().unwrap_or_default().to_string())
}
