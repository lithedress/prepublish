use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait Post:
    'static + Sized + Send + Sync + Unpin + Default + DeserializeOwned + JsonSchema
{
}

#[derive(JsonSchema)]
#[derive(Deserialize)]
#[derive(Default)]
pub struct BlankSubmitted;
impl Post for BlankSubmitted {}

pub trait Postable: 'static + Sized + Send + Sync + Unpin + From<Self::Post> {
    type Post: Post;
}

pub trait Patch:
    'static + Sized + Send + Sync + Unpin + Serialize + DeserializeOwned + JsonSchema
{
}

#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone)]
pub struct BlankPatch;

impl Patch for BlankPatch {}

/// Used to build patch request body.
pub trait Patchable: 'static + Sized + Send + Sync {
    type Patch: Patch;
}

impl Patchable for () {
    type Patch = BlankPatch;
}

/// Used to build response body.
///
/// Should not contain any database dedicated data type,
pub trait View:
    'static + Sized + Send + Sync + Unpin + Serialize + JsonSchema + From<Self::Object>
{
    type Object: 'static + Sized + Send + Sync + Unpin;
}

impl View for () {
    type Object = Self;
}

pub trait Viewable: 'static + Sized + Send + Sync + Unpin {
    type View: View<Object = Self>;
}

impl Viewable for () {
    type View = ();
}

/// Used for hardcode, such as database entity names and url names.
///
/// Don't use for UIs and documents.
pub trait Countable {
    fn singular() -> &'static str;
    fn plural() -> &'static str;
}
