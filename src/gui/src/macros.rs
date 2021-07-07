#[macro_export]
macro_rules! d {
    ( $x:expr, px ) => {
        stretch::style::Dimension::Points($x as f32)
    };
    ( $x:expr, % ) => {
        stretch::style::Dimension::Percent(($x as f32) / 100.0)
    };
    ( auto ) => {
        stretch::style::Dimension::Auto
    };
    ( undef ) => {
        stretch::style::Dimension::Undefined
    };
    ( $x:expr ) => {
        d!($x, px)
    };
}

#[macro_export]
macro_rules! rect {
    ( $all:expr ) => {
        stretch::geometry::Rect {
            top: $all,
            bottom: $all,
            start: $all,
            end: $all,
        }
    };
    ( $vert:expr, $hor:expr ) => {
        stretch::geometry::Rect {
            top: $vert,
            bottom: $vert,
            start: $hor,
            end: $hor,
        }
    };
    ( $top:expr, $right:expr, $bottom:expr, $left:expr ) => {
        stretch::geometry::Rect {
            top: $top,
            bottom: $bottom,
            start: $left,
            end: $right,
        }
    };
}

// I'm not sure how to make this cleaner
#[macro_export]
macro_rules! size {
    ( $x:expr, %; $y:expr, % ) => {
        stretch::geometry::Size {
            width: d!($x, %),
            height: d!($y, %),
        }
    };
    ( $x:expr, px; $y:expr, % ) => {
        stretch::geometry::Size {
            width: d!($x, px),
            height: d!($y, %),
        }
    };
    ( $x:expr, %; $y:expr, px ) => {
        stretch::geometry::Size {
            width: d!($x, %),
            height: d!($y, px),
        }
    };
    ( $x:expr, px; $y:expr, px ) => {
        stretch::geometry::Size {
            width: d!($x, px),
            height: d!($y, px),
        }
    };

    ( $x:expr, %; $y:ident ) => {
        stretch::geometry::Size {
            width: d!($x, %),
            height: d!($y),
        }
    };
    ( $x:expr, px; $y:ident ) => {
        stretch::geometry::Size {
            width: d!($x, px),
            height: d!($y),
        }
    };
    ( $x:ident; $y:expr, px ) => {
        stretch::geometry::Size {
            width: d!($x),
            height: d!($y, px),
        }
    };
    ( $x:ident; $y:expr, % ) => {
        stretch::geometry::Size {
            width: d!($x),
            height: d!($y, %),
        }
    };

    ( $x:ident; $y:ident ) => {
        stretch::geometry::Size {
            width: d!($x),
            height: d!($y),
        }
    };
}

#[macro_export]
macro_rules! style {
    ( $( $key:ident => $value:expr ),* ) => {
            stretch::style::Style {
                $(
                    $key: $value,
                )*
                ..Default::default()
            }
        };
    }

#[macro_export]
macro_rules! rgb {
    ( $r:expr, $g:expr, $b:expr ) => {
        imgui::ImColor32::from_rgb($r, $g, $b)
    };
}

#[macro_export]
macro_rules! rgba {
    ( $r:expr, $g:expr, $b:expr, $a:expr ) => {
        imgui::ImColor32::from_rgba($r, $g, $b, $a)
    };
}

#[macro_export]
macro_rules! rgbf {
    ( $r:expr, $g:expr, $b:expr ) => {
        imgui::ImColor32::from_rgb_f32s($r as f32, $g as f32, $b as f32)
    };
}

#[macro_export]
macro_rules! rgbaf {
    ( $r:expr, $g:expr, $b:expr, $a:expr ) => {
        imgui::ImColor32::from_rgba_f32s($r as f32, $g as f32, $b as f32, $a as f32)
    };
}
