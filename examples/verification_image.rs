// MIT/Apache2 License

use chalkboard::{Color, FillRule, Surface};

pub fn verification_image<S: Surface + ?Sized>(surface: &mut S) -> chalkboard::Result {
    surface.set_stroke(Color::BLACK)?;
    surface.set_fill(FillRule::SolidColor(
        Color::new(0.0, 0.0, 1.0, 1.0).unwrap(),
    ))?;
    surface.set_line_width(8)?;

    surface.fill_rectangle(50.0, 50.0, 150.0, 100.0)?;
    surface.draw_rectangle(50.0, 50.0, 150.0, 100.0)?;

    surface.set_fill(FillRule::SolidColor(
        Color::new(0.0, 1.0, 0.0, 1.0).unwrap(),
    ))?;

    surface.fill_ellipse(300.0, 200.0, 50.0, 75.0)?;
    surface.draw_ellipse(300.0, 200.0, 50.0, 75.0)?;
    surface.draw_line(250.0, 125.0, 350.0, 275.0)?;

    surface.flush()
}
