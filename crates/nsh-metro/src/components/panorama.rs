//! The panorama: a wide title over a strip of sections that wraps around;
//! the title moves at a fraction of the sections' speed (parallax).
//!
//! The whole strip's position is one animated value: changing section
//! settles it, and the title follows it through a native interpolation.
//! On first show both slide in from the right (the Metro intro).

use std::cell::Cell;
use std::rc::Rc;

use nsh_core::prelude::*;

use crate::components::text::label;

use crate::{
    chrome::PAGE_MARGIN,
    components::pager,
    motion::EXP6_OUT,
    pivot::{PaneBuilder, SelectHandler},
    theme::{FONT_FAMILY, LIGHT, palette, reduced_motion, size, u},
};

pub struct PanoramaSection {
    header: SharedString,
    content: PaneBuilder,
}

pub struct Panorama {
    title: SharedString,
    sections: Vec<PanoramaSection>,
    selected: Reactive<usize>,
    section_width: f32,
    on_select: Option<SelectHandler>,
}

pub fn panorama(title: impl Into<SharedString>, selected: impl IntoReactive<usize>) -> Panorama {
    Panorama { title: title.into(), sections: Vec::new(), selected: selected.into_reactive(), section_width: 420., on_select: None }
}

impl Panorama {
    pub fn section<E: IntoElement>(mut self, header: impl Into<SharedString>, content: impl Fn() -> E + 'static) -> Self {
        self.sections.push(PanoramaSection { header: header.into(), content: Rc::new(move || content().into_element()) });
        self
    }
    pub fn section_width(mut self, w: f32) -> Self {
        self.section_width = w;
        self
    }
    pub fn on_select(mut self, f: impl Fn(usize) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

/// The title moves this fraction of the sections' distance.
pub const PARALLAX: f32 = 0.4;
const SETTLE: f32 = 450.;
const INTRO: f32 = 650.;
const INTRO_SECTIONS: f32 = 360.;
const INTRO_TITLE: f32 = INTRO_SECTIONS * PARALLAX;

/// Where section `ix` sits (in section widths) when the strip is at `pos`:
/// with wrapping, the first section follows the last.
pub fn section_slot(ix: usize, pos: f32, len: usize, wraps: bool) -> f32 {
    if !wraps || len == 0 {
        return ix as f32;
    }
    let l = len as f32;
    ix as f32 + l * ((pos + 0.45 - ix as f32) / l).round()
}

/// The strip position to settle on for `selected`, the short way from `from`.
pub fn settle_target(from: f32, selected: usize, len: usize, wraps: bool) -> f32 {
    if !wraps || len == 0 {
        return selected as f32;
    }
    let l = len as f32;
    selected as f32 + l * ((from - selected as f32) / l).round()
}

impl IntoElement for Panorama {
    fn into_element(self) -> AnyElement {
        let len = self.sections.len();
        let wraps = len >= 3;
        let sw = self.section_width;
        let selected = self.selected;
        let first = untracked(|| selected.get()).min(len.saturating_sub(1));
        // Strip position, in sections; `slots_at` is where the sections were
        // last laid out for (they wrap around it).
        let pos = animated_value(first as f32);
        let slots_at = signal(first as f32);
        let at = Rc::new(Cell::new(first as f32));
        let last = Rc::new(Cell::new(first));
        {
            let (at, last) = (at.clone(), last.clone());
            effect(move || {
                let sel = selected.get().min(len.saturating_sub(1));
                if last.replace(sel) == sel {
                    return;
                }
                untracked(|| {
                    let from = at.get();
                    let to = settle_target(from, sel, len, wraps);
                    // Lay the sections out around the whole move, then glide.
                    slots_at.set((from + to) / 2.);
                    at.set(to);
                    if reduced_motion() {
                        pos.set(to);
                    } else {
                        let end = pos.timing(to, SETTLE, EXP6_OUT);
                        let at = at.clone();
                        spawn(async move {
                            if end.await && wraps {
                                // Back into 0..len, invisibly.
                                let l = len as f32;
                                let norm = to.rem_euclid(l);
                                if norm != to && at.get() == to {
                                    at.set(norm);
                                    pos.set(norm);
                                    slots_at.set(norm);
                                }
                            }
                        });
                    }
                });
            });
        }

        let intro = animated_value(if reduced_motion() { 0. } else { 1. });
        if !reduced_motion() {
            let _ = intro.timing(0., INTRO, EXP6_OUT);
        }

        let title = label(SharedString::from(self.title.to_lowercase()))
            .font_family(FONT_FAMILY)
            .text_size(u(size::PANORAMA))
            .font_weight(LIGHT)
            .number_of_lines(1u32)
            .text_color(move || palette().foreground);
        let title = view()
            .absolute()
            .top(u(-18.))
            .left(u(PAGE_MARGIN - 6.))
            .h(u(140.))
            .bind_translate_x(&pos.mul(-sw * PARALLAX).add(&intro.mul(INTRO_TITLE)))
            .child(title);

        let on_select = self.on_select.clone();
        let sections = Rc::new(self.sections);
        let strip = {
            let sections = sections.clone();
            move || {
                let around = slots_at.get();
                view()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(u(PAGE_MARGIN))
                    .bind_translate_x(&pos.mul(-sw).add(&intro.mul(INTRO_SECTIONS)))
                    .children(sections.iter().enumerate().map(|(ix, s)| {
                        let slot = section_slot(ix, around, len, wraps);
                        let on_select = on_select.clone();
                        let section = view()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left(u(slot * sw))
                            .w(u(sw))
                            .pr(u(PAGE_MARGIN))
                            .child(
                                label(SharedString::from(s.header.to_lowercase()))
                                    .font_family(FONT_FAMILY)
                                    .text_size(u(size::EXTRA_LARGE + 4.))
                                    .font_weight(LIGHT)
                                    .pb(u(10.))
                                    .text_color(move || palette().foreground),
                            )
                            .child(view().flex_1().min_h(px(0.)).child((s.content)()));
                        match on_select {
                            Some(f) => section.cursor_pointer().on_press(move || f(ix)).into_element(),
                            None => section.into_element(),
                        }
                    }))
                    .into_element()
            }
        };
        let root = view()
            .relative()
            .size_full()
            .overflow_hidden()
            .child(title)
            .child(view().absolute().top(u(132.)).bottom_0().left_0().right_0().child(dynamic(strip)));
        match self.on_select.filter(|_| len > 1) {
            Some(f) => pager::pageable(
                root,
                Rc::new(move |step| {
                    let sel = last.get();
                    let next = if step > 0 { sel + 1 } else { sel + len - 1 };
                    if wraps || (step > 0 && next < len) || (step < 0 && sel > 0) {
                        f(next % len);
                    }
                }),
            )
            .into_element(),
            None => root.into_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_section_follows_the_last() {
        assert_eq!(section_slot(0, 2., 3, true), 3.);
        assert_eq!(section_slot(1, 2., 3, true), 1.);
        assert_eq!(section_slot(2, 0., 3, true), -1.);
        assert_eq!(section_slot(0, 2., 3, false), 0.);
    }

    #[test]
    fn settling_takes_the_short_way_round() {
        assert_eq!(settle_target(2., 0, 3, true), 3.);
        assert_eq!(settle_target(0., 2, 3, true), -1.);
        assert_eq!(settle_target(0., 2, 3, false), 2.);
    }
}
