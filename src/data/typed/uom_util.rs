use uom::si::{
    angle::{degree, minute, radian, revolution, second},
    length::{foot, kilometer, meter, nautical_mile},
    velocity::{
        foot_per_minute, foot_per_second, kilometer_per_hour, knot, meter_per_second, mile_per_hour,
    },
};

macro_rules! impl_uom_convert {
    (
     [$(
         $f:tt, $uom_type:path, [$(
            ($fn:ident, $unit:ty)
        ),*]
    ),*]) => {
        pastey::paste!{
            /// uom -> float
            pub trait FromUom: Copy
            {
                $($(
                    #[inline]
                    fn [<$fn _ $f>](self) -> $f
                    where
                        Self: Copy + Into<uom::si::$f::$uom_type>,
                    {
                        self.into().get::<$unit>()
                    }
                )*)*
            }
            $( impl FromUom for uom::si::$f::$uom_type {} )*

            /// float -> uom
            pub trait IntoUom: Copy
            {
                $($(
                    #[inline]
                    fn [<$fn _ $f>](self) -> uom::si::$f::$uom_type where Self: Copy + Into<$f>,
                    {
                        <uom::si::$f::$uom_type>::new::<$unit>(self.into())
                    }
                )*)*
            }

            impl<T: Into<f64>> IntoUom for T where T: Copy {}
        }
    };
}

impl_uom_convert! {
    [
        f64, Angle, [
            (radians, radian),
            (degrees, degree),
            (seconds, second),
            (minutes, minute),
            (revolutions, revolution)
        ],
        f64, Length, [
            (meters, meter),
            (kilometers, kilometer),
            (feet, foot),
            (nautical_miles, nautical_mile)
        ],
        f64, Velocity, [
            (meters_per_second, meter_per_second),
            (kilometers_per_hour, kilometer_per_hour),
            (feet_per_second, foot_per_second),
            (feet_per_minute, foot_per_minute),
            (miles_per_hour, mile_per_hour),
            (knots, knot)
        ],
        f32, Angle, [
            (radians, radian),
            (degrees, degree),
            (seconds, second),
            (minutes, minute),
            (revolutions, revolution)
        ],
        f32, Length, [
            (meters, meter),
            (kilometers, kilometer),
            (feet, foot),
            (nautical_miles, nautical_mile)
        ],
        f32, Velocity, [
            (meters_per_second, meter_per_second),
            (kilometers_per_hour, kilometer_per_hour),
            (feet_per_second, foot_per_second),
            (feet_per_minute, foot_per_minute),
            (miles_per_hour, mile_per_hour),
            (knots, knot)
        ]
    ]
}
