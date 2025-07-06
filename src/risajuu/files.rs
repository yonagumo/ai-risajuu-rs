#![allow(dead_code)]

use reqwest::multipart;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::Read;

/* サポートする形式
 * ドキュメント形式 (application/)
 * pdf, x-javascript, x-python
 * ドキュメント形式 (text/)
 * javascript, x-python, plain, html, css, md, csv, xml, rtf
 * 画像形式 (image/)
 * png, jpeg, webp, heic, heif
 * 動画形式 (video/)
 * mp4, mpeg, mov, avi, x-flv, mpg, webm, wmv, 3gpp
 * オーディオ形式 (audio/)
 * wav, mp3, aiff, aac, ogg, flac
 */

pub async fn upload_file(api_key: &str, mime: &str, file_path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    upload_bytes(api_key, mime, buf).await
}

pub async fn upload_bytes(api_key: &str, mime: &str, buf: Vec<u8>) -> Result<String, Box<dyn Error>> {
    let url = "https://generativelanguage.googleapis.com/upload/v1beta/files?key=".to_string() + api_key;

    let part = multipart::Part::bytes(buf).file_name("upload.bin").mime_str(mime)?;

    let form = multipart::Form::new().part("file", part);

    let client = reqwest::Client::new();
    let response: Value = client.post(&url).multipart(form).send().await?.json().await?;

    //println!("uploaded: {:?}", res);

    // URIを返す
    Ok(response["file"]["uri"].as_str().ok_or("response[\"file\"][\"url\"] is None")?.to_string())
}
