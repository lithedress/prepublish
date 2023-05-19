use axum::{
    body::StreamBody,
    extract::Multipart,
    http::{header, HeaderName},
};
use futures_codec::{BytesCodec, FramedRead};
use futures_util::{Stream, TryStreamExt};
use mongo::{gridfs, oid::ObjectId, MongoDatabase};

use super::err::{Error, Result};

pub(crate) async fn upload_files(
    db: MongoDatabase,
    multipart: &mut Multipart,
) -> Result<Vec<ObjectId>> {
    let mut file_ids = Vec::new();
    let bucket = db.gridfs_bucket(None);
    while let Some(field) = multipart.next_field().await.map_err(Error::from)? {
        file_ids.push(
            gridfs::upload(
                &bucket,
                field
                    .file_name()
                    .ok_or(Error::BadReqest("file name needed".to_string()))?
                    .to_string(),
                field.size_hint().1.map(|u| u as u32),
                field.content_type().map(ToString::to_string),
                &field.bytes().await.map_err(Error::from)?,
            )
            .await
            .map_err(Error::from)?
            .ok_or(Error::NotFound("inserted id not found".to_string()))?,
        );
    }
    Ok(file_ids)
}

pub(crate) async fn download_file(
    db: MongoDatabase,
    id: ObjectId,
) -> Result<(
    [(HeaderName, String); 3],
    StreamBody<impl Stream<Item = std::io::Result<Vec<u8>>> + Sized>,
)> {
    let bucket = db.gridfs_bucket(None);
    let (file, stream) = gridfs::download(bucket, id).await?;
    let file = file.ok_or(Error::NotFound("no file".to_string()))?;
    let stream = FramedRead::new(stream, BytesCodec).map_ok(|b| b.to_vec());
    Ok((
        [
            (
                header::CONTENT_DISPOSITION,
                file.filename
                    .map(|nm| format!("attachment; filename=\"{}\"", nm))
                    .unwrap_or_default(),
            ),
            (header::CONTENT_LENGTH, file.length.to_string()),
            (
                header::CONTENT_TYPE,
                file.metadata
                    .and_then(|md| md.get("Content-Type").map(ToString::to_string))
                    .unwrap_or_default(),
            ),
        ],
        StreamBody::new(stream),
    ))
}
