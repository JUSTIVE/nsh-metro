use nsh_core::{element::desktop::window, prelude::*, TransformKind, element::TransformOrigin};
use nsh_platform_macos as platform;

const EXP6_OUT: Easing = Easing::Bezier(0.15, 0.95, 0.4, 1.0);

#[component]
pub fn App() -> impl IntoElement {
    let angles: Vec<AnimatedValue> = (0..6).map(|_| animated_value(90.0)).collect();
    let anim = Animation::sequence([
        Animation::stagger(40.0, angles.iter().map(|a| Animation::timing(a, 0.0, Timing::new(350.0, EXP6_OUT)))),
        Animation::delay(800.0),
        Animation::stagger(45.0, angles.iter().map(|a| Animation::timing(a, 90.0, Timing::new(250.0, Easing::Bezier(0.6, 0.0, 0.85, 0.05))))),
        Animation::delay(500.0),
    ])
    .looped(None);
    let _ = anim.start();
    let tiles = move || angles.clone().into_iter().enumerate().map(|(i, a)| {
        let (x, y) = ((i % 2) as f32 * 160.0, (i / 2) as f32 * 160.0);
        view()
            .absolute()
            .left(px(x))
            .top(px(y))
            .size(px(150.0))
            .bg(rgb(0x0050EF))
            .transform_origin(TransformOrigin { x: px(-(x + 24.0)).into(), y: relative(0.5).into(), z: Points::ZERO })
            .bind_transform(TransformKind::RotateY, &a)
            .child(text(format!("tile {i}")).text_color(rgb(0xffffff)))
    }).collect::<Vec<_>>();
    window().id("main").content(move || {
        view().flex_1().bg(rgb(0x000000)).p(px(24.0)).perspective(px(1000.0)).child(
            view().relative().size(px(320.0)).children(tiles()),
        )
    })
}

fn main() -> impl std::process::Termination {
    platform::run_app(
        platform::AppSource::Rust(platform::Root::of::<App>()),
        platform::AppOptions { title: "nsh hello".to_owned(), size: (480, 640), ..Default::default() },
    )
}
