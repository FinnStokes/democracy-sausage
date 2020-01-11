type Colour = [f32; 4];

pub fn interpolate_colour(colours: &[(Colour, f32)], point: f32) -> Colour {
    let lower = colours.iter().filter(|(_, x)| *x < point).last();
    let upper = colours.iter().filter(|(_, x)| *x >= point).next();

    match (lower, upper) {
        (None, None) => panic!("Gradient missing reference points"),
        (Some((colour, _)), None) => *colour,
        (None, Some((colour, _))) => *colour,
        (Some((lc, l)), Some((uc, u))) => {
            let s = (point - l) / (u - l);
            [
                lc[0] * (1.0 - s) + uc[0] * s,
                lc[1] * (1.0 - s) + uc[1] * s,
                lc[2] * (1.0 - s) + uc[2] * s,
                lc[3] * (1.0 - s) + uc[3] * s,
            ]
        },
    }
}
