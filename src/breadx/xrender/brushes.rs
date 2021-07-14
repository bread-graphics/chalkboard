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
use breadx::{
    auto::render::Repeat,
    prelude::*,
    render::{Pictformat, PictureParameters, RenderDisplay},
};
use std::collections::hash_map::{Entry, HashMap};

/// A container for "brushes" (e.g. things we use to composite against the mask) that cleans itself up if it
/// allocates too much memory.
pub(crate) struct Brushes {
    // the actual map of brushes
    brushes: HashMap<FillRuleKey, Collected<MaybePixmapPicture>>,
    // true is we are not doing GC anymore
    disable_gc: bool,
}

/// Number of brushes we allocate before we start collecting garbage. This should be high enough that most use
/// cases shouldn't hit it, but shouldn't be too low so that the X server won't run out of memory.
const BRUSH_LIMIT: usize = 1 << 12;

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
    pub(crate) fn fill<D: Display>(
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
                FillRuleKey::LinearGradient(grad, angle, rect) => {
                    // figure out the dimensions for the gradient
                    let Rectangle { x1, y1, x2, y2 } = rect;
                    let width = (x2 - x1).abs(); let height = (y2 - y1).abs();
                    let (p1, p2) = rectangle_angle(width as f64, height as f64, *angle);
                    let (stops, color) = gradient_to_stops_and_color(grad);
                    
                    // create the gradient proper
                    let grad = dpy.create_linear_gradient(p1, p2, stops, color)?;
                    v.insert(grad);
                    Ok(grad)
                }
            },
        }
    }

    #[inline]
    fn gc<D: Display>(&mut self, dpy: &mut D) -> crate::Result {}
}
