//               Copyright John Nunley, 2022.
// Distributed under the Boost Software License, Version 1.0.
//       (See accompanying file LICENSE or copy at
//         https://www.boost.org/LICENSE_1_0.txt)

use super::{PathBuffer, Region};

/// The clipping region for a given operation.
pub struct Clip {
    // the clip is a combination of the path (closed) and the region
    path: PathBuffer,
    region: Region<f32>,
}
