//               Copyright John Nunley, 2022.
// Distributed under the Boost Software License, Version 1.0.
//       (See accompanying file LICENSE or copy at
//         https://www.boost.org/LICENSE_1_0.txt)

use num_traits::Float;

pub(crate) fn approx_eq<Num: Float>(a: Num, b: Num) -> bool {
    (a - b).abs() < Num::epsilon()
}
