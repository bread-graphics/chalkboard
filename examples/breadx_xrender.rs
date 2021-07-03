// MIT/Apache2 License

#[cfg(all(unix, feature = "xrender"))]
use breadx::{prelude::*, render::RenderDisplay, DisplayConnection, Event, EventMask};

#[cfg(all(unix, feature = "xrender"))]
use chalkboard::{
    breadx::{RenderBreadxSurface, RenderResidual},
    Color, FillRule, Result, Surface,
};

#[cfg(all(unix, feature = "xrender"))]
fn main() -> Result {
    env_logger::init();

    let mut width = 640u16;
    let mut height = 400u16;

    let mut conn = DisplayConnection::create(None, None)?;
    let window = conn.create_simple_window(
        conn.default_root(),
        0,
        0,
        width,
        height,
        0,
        conn.default_black_pixel(),
        conn.default_white_pixel(),
    )?;
    window.set_title(&mut conn, "XRender Chalkboard Example")?;
    window.map(&mut conn)?;
    window.set_event_mask(&mut conn, EventMask::EXPOSURE | EventMask::STRUCTURE_NOTIFY)?;

    let wdw = conn.intern_atom_immediate("WM_DELETE_WINDOW", false)?;
    window.set_wm_protocols(&mut conn, &[wdw])?;

    // initialize xrender
    let mut conn = RenderDisplay::new(conn, 0, 10).map_err(|(_, e)| e)?;

    // create a picture for the window
    let attrs = window.window_attributes_immediate(&mut conn)?;
    let visual = attrs.visual;
    let visual = conn.visual_id_to_visual(visual).unwrap();
    let window_format = conn.find_visual_format(visual).unwrap();

    let picture = conn.create_picture(window, window_format, Default::default())?;
    let mut residual: Option<RenderResidual> = None;

    loop {
        match conn.wait_for_event()? {
            Event::ClientMessage(cme) => {
                let atom = cme.data.longs().get(0).copied();
                if atom == Some(wdw.xid) {
                    if let Some(residual) = residual.take() {
                        residual.free(&mut conn)?;
                    }
                    break;
                }
            }
            Event::ConfigureNotify(cc) => {
                width = cc.width;
                height = cc.height;
            }
            Event::Expose(_) => {
                // create the surface
                let mut surface = match residual.take() {
                    None => RenderBreadxSurface::new(&mut conn, picture, window, width, height)?,
                    Some(residual) => RenderBreadxSurface::from_residual(
                        &mut conn, picture, window, width, height, residual,
                    )?,
                };

                // draw some shapes
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

                // cache the residual
                residual = Some(surface.into_residual());
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(not(all(unix, feature = "xrender")))]
fn main() {
    println!("In order to run the xrender example, xrender needs to be enabled");
}
