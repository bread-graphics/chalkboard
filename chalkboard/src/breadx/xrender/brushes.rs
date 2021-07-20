// MIT/Apache2 License

//! It is useful to have a hash map associating different types of fill with different brushes, so that we can
//! cache commonly used brushes and reduce communication to the server, which should lead to an overall speedup.
//! However, consider the use case where a malicious or incompetent user creates and uses several independent
//! types of gradients, or a truly massive GUI app that uses several different types of switches and buttons.
//! This will create several thousand independent gradients and brushes, which will flood the X server with
//! gradients and lead to an OOM situation in the worst case.
//!
//! In order to defeat this worst-case scenario, we've created what's essentially a "garbage collected" hash map
//! of brushes. The amount of usage each brush gets is tracked.
//!
//! TODO: finish the algorithm

use super::{FillRuleKey, MaybePixmapPicture};
use crate::{Angle, gradient::Gradient};
use approx::abs_diff_eq;
use breadx::{
    auto::{
        render::{Repeat, Transform, Pointfix},
        xproto::Drawable,
    },
    prelude::*,
    render::{Pictformat, Picture, PictureParameters, RenderDisplay, double_to_fixed},
};
use std::collections::hash_map::{Entry, HashMap};
use tinyvec::TinyVec;

/// A container for "brushes" (e.g. things we use to composite against the mask) that cleans itself up if it
/// allocates too much memory.
#[derive(Debug)]
pub(crate) struct Brushes {
    // the actual map of brushes
    brushes: HashMap<FillRuleKey, Collected<MaybePixmapPicture>>,
    // true is we are not doing GC anymore
    disable_gc: bool,
}

/// Number of brushes we allocate before we start collecting garbage. This should be high enough that most use
/// cases shouldn't hit it, but shouldn't be too low so that the X server won't run out of memory.
const BRUSH_LIMIT: usize = 1 << 12;

#[derive(Debug)]
struct Collected<T> {
    usage: usize,
    inner: T,
}

impl Brushes {
    #[inline]
    pub(crate) fn new() -> Brushes {
        Brushes {
            brushes: HashMap::new(),
            disable_gc: false,
        }
    }

    #[inline]
    pub(crate) fn free<D: Display + ?Sized>(self, display: &mut D) -> crate::Result {
        self.brushes
            .into_iter()
            .try_for_each(|(_, Collected { inner, .. })| inner.free(display))
    }

    #[inline]
    pub(crate) fn fill<D: Display + ?Sized>(
        &mut self,
        dpy: &mut RenderDisplay<D>,
        parent: Drawable,
        parent_depth: u8,
        parent_format: Pictformat,
        key: FillRuleKey,
    ) -> crate::Result<Picture> {
        match self.brushes.entry(key) {
            Entry::Occupied(o) => {
                // increment the usage count of the instance
                let usage = o.get().usage.saturating_add(1);
                o.get_mut().usage = usage;
                Ok(o.get().inner.picture())
            }
            Entry::Vacant(v) => match v.key() {
                FillRuleKey::Color(clr) => {
                    // create a 1x1 pixmap with the same format and depth as the window
                    let pm = dpy.create_pixmap(parent, 1, 1, parent_depth)?;
                    // create an accompanying picture
                    let pmp = dpy.create_picture(
                        pm,
                        parent_format,
                        PictureParameters {
                            repeat: Some(Repeat::Normal),
                            ..Default::default()
                        },
                    )?;
                    // insert that
                    v.insert(Collected {
                        usage: 0,
                        inner: MaybePixmapPicture {
                            picture: pmp,
                            pixmap: Some(pm),
                        },
                    });
                    Ok(pmp)
                }
                FillRuleKey::LinearGradient(grad, angle, width, height) => {
                    // figure out the dimensions for the gradient
                    let (p1, p2) = rectangle_angle(*width as f64, *height as f64, *angle);
                    let (stops, color) = gradient_to_stops_and_color(grad);

                    // create the gradient proper
                    let grad = dpy.create_linear_gradient(p1, p2, stops, color)?;

                    v.insert(Collected {
                        usage: 0,
                        inner: grad.into(),
                    });
                    Ok(grad)
                }
                FillRuleKey::RadialGradient(grad, width, height) => {
                    // get the dimensions of the radius gradient
                    let radius = double_to_fixed(*width as f64);
                    let scaling = (*height as f64) / (*width as f64);

                    // get the center point
                    let c = radius / 2;
                    let cp = Pointfix { x: c, y: c };

                    let (stops, color) = gradient_to_stops_and_color(grad);

                    // create the basic radial gradient
                    let radial =
                        dpy.create_radial_gradient(cp.clone(), cp, 0, radius, stops, color)?;

                    // apply a transform that scales it to fit the width/height
                    if width != height {
                        radial.set_transform(
                            dpy,
                            Transform {
                                matrix11: 1 << 16,
                                matrix22: double_to_fixed(scaling),
                                matrix33: 1 << 16,
                                ..Default::default()
                            },
                        )?;
                    }

                    v.insert(Collected {
                        usage: 0,
                        inner: radial.into(),
                    });
                    Ok(radial)
                }
                FillRuleKey::ConicalGradient(grad, width, height) => {
                    // get the dimensions of the radius gradient
                    let radius = double_to_fixed(*width as f64);
                    let scaling = (*height as f64) / (*width as f64);

                    // get the center point
                    let c = radius / 2;
                    let cp = Pointfix { x: c, y: c };

                    let (stops, color) = gradient_to_stops_and_color(grad);

                    // create the basic conical gradient
                    let conical = dpy.create_conical_gradient(cp, 0, stops, color)?;

                    // apply a transform that scales it to fit the width/height
                    if width != height {
                        conical.set_transform(
                            dpy,
                            Transform {
                                matrix11: 1 << 16,
                                matrix22: double_to_fixed(scaling),
                                matrix33: 1 << 16,
                                ..Default::default()
                            },
                        )?;
                    }

                    v.insert(Collected {
                        usage: 0,
                        inner: conical.into(),
                    });
                    Ok(conical)
                }
            },
        }
    }

    /*#[inline]
    fn gc<D: Display>(&mut self, dpy: &mut D) -> crate::Result {}*/
}

// helper function to get stops and color from a gradient
#[inline]
fn gradient_to_stops_and_color(grad: &Gradient) -> (TinyVec<[Fixed; 6]>, TinyVec<[XrColor; 3]>) {
    grad.iter()
        .map(|r| {
            (
                double_to_fixed(r.position.into_inner().into()),
                cvt_color(r.color),
            )
        })
        .unzip()
}

// helper function to get points on the rectangle corresponding to angles
#[inline]
fn rectangle_angle(width: f64, height: f64, angle: Angle) -> (Pointfix, Pointfix) {
    // fast paths
    if angle.approx_eq(Angle::ZERO) || angle.approx_eq(Angle::FULL_CIRCLE) {
        let h2 = double_to_fixed(height / 2.0);
        return (
            Pointfix { x: 0, y: h2 },
            Pointfix {
                x: double_to_fixed(width),
                y: h2,
            },
        );
    } else if angle.approx_eq(Angle::QUARTER_CIRCLE) {
        let w2 = double_to_fixed(width / 2.0);
        return (
            Pointfix { x: w2, y: 0 },
            Pointfix {
                x: w2,
                y: double_to_fixed(height),
            },
        );
    } else if angle.approx_eq(Angle::HALF_CIRCLE) {
        let h2 = double_to_fixed(height / 2.0);
        return (
            Pointfix { x: 0, y: h2 },
            Pointfix {
                x: double_to_fixed(width),
                y: h2,
            },
        );
    } else if angle.approx_eq(Angle::THREE_QUARTERS_CIRCLE) {
        let w2 = double_to_fixed(width / 2.0);
        return (
            Pointfix {
                x: w2,
                y: double_to_fixed(height),
            },
            Pointfix { x: w2, y: 0 },
        );
    }

    // slow path: calculate a point going from the center of the rectangle to the edges. we can do this with
    // the knowledge that the slope, which we can calculate via atan(angle), is y/x. given the center, we can use
    // this to calculate the x at y=height and the y at x=width, and figure out which one fits
    // then do the same at y = 0 and x = 0
    let slope = angle.radians().atan() as f64;
    let xc = width / 2.0;
    let yc = height / 2.0;

    let mut calc_point_at = move |xfix: f64, yfix: f64| -> Pointfix {
        // (y - y1) = m*(x - x1) -> y = y1 + m*(x - x1), where x1 and y1 are the center and x = width or 0
        let y_est = yc + (slope * (xfix - xc));
        // x = x1 + (y - y1)/m
        let x_est = xc + ((yfix - yc) / slope);
        // calculate the results of the estimate
        let y_est_x = xc + ((yfix - y_est) / slope);
        let x_est_y = yc + (slope * (xfix - x_est));

        // one of these estimates will be larger than the rectangle proper, so account for that and choose which
        // one works
        if y_est_x > width {
            Pointfix {
                x: double_to_fixed(x_est),
                y: double_to_fixed(x_est_y),
            }
        } else {
            Pointfix {
                x: double_to_fixed(y_est_x),
                y: double_to_fixed(y_est),
            }
        }
    };

    (calc_point_at(0.0, 0.0), calc_point_at(width, height))
}
