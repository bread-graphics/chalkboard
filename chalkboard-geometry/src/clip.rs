// BSL 1.0 License

use super::{PathBuffer, Region};

/// The clipping region for a given operation.
pub struct Clip {
    // the clip is a combination of the path (closed) and the region
    path: PathBuffer,
    region: Region<f32>,
}
