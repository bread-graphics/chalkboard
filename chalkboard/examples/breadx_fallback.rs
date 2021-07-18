// MIT/Apache2 License

#[cfg(all(unix, feature = "breadx"))]
use breadx::{prelude::*, DisplayConnection, Event, EventMask, GcParameters};

#[cfg(all(unix, feature = "breadx"))]
use chalkboard::{breadx::FallbackBreadxSurface, Color, FillRule, Result, Surface};

#[cfg(all(unix, feature = "breadx"))]
use std::{collections::HashMap, mem};

#[cfg(all(unix, feature = "breadx"))]
fn main() -> Result {
    env_logger::init();

    // create the connection and the window
    let mut conn = DisplayConnection::create(None, None)?;
    let root = conn.default_root();
    let black = conn.default_black_pixel();
    let white = conn.default_white_pixel();
    let win = conn.create_simple_window(root, 0, 0, 640, 480, 1, black, white)?;

    // show the window and start processing events
    let wdw = conn.intern_atom_immediate("WM_DELETE_WINDOW", false)?;
    win.set_wm_protocols(&mut conn, &[wdw])?;
    win.set_event_mask(&mut conn, EventMask::EXPOSURE)?;
    win.map(&mut conn)?;

    // allocate a gc
    let gc = conn.create_gc(
        win,
        GcParameters {
            graphics_exposures: Some(1),
            ..Default::default()
        },
    )?;

    // create the colormap
    let mut colormap = Some(HashMap::new());

    loop {
        match conn.wait_for_event()? {
            Event::ClientMessage(cme) => {
                // find out if we need to quit
                let atom = cme.data.longs().get(0).copied();

                if atom == Some(wdw.xid) {
                    break;
                }
            }
            Event::Expose(ee) => {
                // time to paint!
                let mut surface = FallbackBreadxSurface::with_cached_colormap(
                    &mut conn,
                    win,
                    gc,
                    colormap.take().unwrap(),
                );

                surface.set_stroke(Color::BLACK)?;
                surface.set_fill(FillRule::SolidColor(
                    Color::new(0.0, 0.0, 1.0, 1.0).unwrap(),
                ))?;
                surface.set_line_width(8)?;

                surface.fill_rectangle(50, 50, 150, 100)?;
                surface.draw_rectangle(50, 50, 150, 100)?;

                surface.set_fill(FillRule::SolidColor(
                    Color::new(0.0, 1.0, 0.0, 1.0).unwrap(),
                ))?;

                surface.fill_ellipse(50, 200, 150, 300)?;
                surface.draw_ellipse(50, 200, 150, 300)?;
                surface.draw_line(50, 200, 150, 250)?;

                surface.flush()?;

                // cache our colormap
                colormap = Some(surface.into_colormap());
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(not(all(unix, feature = "breadx")))]
fn main() {
    println!("Cannot run breadx example unless breadx is enabled");
}