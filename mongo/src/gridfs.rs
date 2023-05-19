use futures_util::{io::AsyncWriteExt, StreamExt};
use mongodm::{
    bson, doc,
    mongo::{
        error, gridfs::FilesCollectionDocument, options::GridFsUploadOptions, GridFsBucket,
        GridFsDownloadStream,
    },
    prelude::ObjectId,
};

pub async fn upload(
    bucket: &GridFsBucket,
    filename: impl AsRef<str>,
    size: impl Into<Option<u32>>,
    content_type: impl Into<Option<String>>,
    content: &[u8],
) -> std::io::Result<Option<ObjectId>> {
    let mut stream = bucket.open_upload_stream(
        filename,
        GridFsUploadOptions::builder()
            .chunk_size_bytes(size)
            .metadata(doc! {"Content-Type": content_type.into()})
            .build(),
    );
    stream.write_all(content).await?;
    stream.close().await?;
    Ok(stream.id().as_object_id())
}

pub async fn download(
    bucket: GridFsBucket,
    id: ObjectId,
) -> error::Result<(Option<FilesCollectionDocument>, GridFsDownloadStream)> {
    Ok((
        bucket
            .find(doc! {"_id": id}, None)
            .await?
            .next()
            .await
            .map_or(Ok(None), |v| v.map(Some))?,
        bucket.open_download_stream(bson!(id)).await?,
    ))
}
