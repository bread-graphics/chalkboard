// MIT/Apache2 License

#[cfg(all(windows, feature = "yaww"))]
use chalkboard::{
    yaww::{YawwGdiSurface, YawwGdiSurfaceResidual},
    Color, FillRule, Result, Surface,
};

#[cfg(all(windows, feature = "yaww"))]
use std::{borrow::Cow, ffi::CStr, ops};

#[cfg(all(windows, feature = "yaww"))]
use yaww::{
    brush::DEFAULT_BRUSH, ClassStyle, Event, ExtendedWindowStyle, GuiThread, SendsDirective,
    ShowWindowCommand, WcFunctions, WindowFunctions, WindowStyle,
};

#[cfg(all(windows, feature = "yaww"))]
const CLASS_NAME: ConstCstr = ConstCstr("TestWindow\0");
#[cfg(all(windows, feature = "yaww"))]
const WINDOW_NAME: ConstCstr = ConstCstr("GDI+ Test\0");

#[cfg(all(windows, feature = "yaww"))]
fn main() -> Result {
    env_logger::init();

    // initialize the GUI thread as well as a window
    let gt = GuiThread::new();
    gt.register_class(
        &*CLASS_NAME,
        None,
        ClassStyle::empty(),
        None,
        None,
        None,
        Some(DEFAULT_BRUSH),
    )?
    .wait()?;
    let win = gt
        .create_window(
            &*CLASS_NAME,
            None,
            Some(Cow::Borrowed(&*WINDOW_NAME)),
            WindowStyle::OVERLAPPED_WINDOW,
            ExtendedWindowStyle::CLIENT_EDGE,
            0,
            0,
            640,
            480,
            None,
            None,
        )?
        .wait()?;
    win.show(&gt, ShowWindowCommand::SHOW)?.wait();

    let mut residual: Option<YawwGdiSurfaceResidual> = None;

    gt.set_event_handler(move |gt, ev| match ev {
        Event::Paint { dc, .. } => {
            let mut surface = match residual.take() {
                None => YawwGdiSurface::new(&gt, dc),
                Some(residual) => YawwGdiSurface::from_residual(&gt, dc, residual),
            };

            // begin painting using the surface
            surface.set_stroke(Color::BLACK).unwrap();
            surface
                .set_fill(FillRule::SolidColor(
                    Color::new(0.0, 0.0, 1.0, 1.0).unwrap(),
                ))
                .unwrap();
            surface.set_line_width(8).unwrap();

            surface.fill_rectangle(50, 50, 150, 100).unwrap();
            surface.draw_rectangle(50, 50, 150, 100).unwrap();

            surface
                .set_fill(FillRule::SolidColor(
                    Color::new(0.0, 1.0, 0.0, 1.0).unwrap(),
                ))
                .unwrap();

            surface.fill_ellipse(50, 200, 150, 300).unwrap();
            surface.draw_ellipse(50, 200, 150, 300).unwrap();
            surface.draw_line(50, 200, 150, 250).unwrap();

            surface.flush().unwrap();

            // cache the residual
            residual = Some(surface.into_residual());

            Ok(())
        }
        Event::Quit => {
            if let Some(residual) = residual.take() {
                residual.free(gt).unwrap();
            }
            Ok(())
        }
        _ => Ok(()),
    });

    gt.main_loop()?;

    Ok(())
}

#[cfg(all(windows, feature = "yaww"))]
struct ConstCstr(&'static str);

#[cfg(all(windows, feature = "yaww"))]
impl ops::Deref for ConstCstr {
    type Target = CStr;

    #[inline]
    fn deref(&self) -> &CStr {
        CStr::from_bytes_with_nul(self.0.as_bytes()).unwrap()
    }
}

#[cfg(not(all(windows, feature = "yaww")))]
fn main() {
    println!("Cannot run yaww example unless yaww is enabled!");
}
