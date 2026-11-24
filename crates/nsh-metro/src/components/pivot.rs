//! The pivot: a row of huge lower-case headers over one pane at a time.
//!
//! Changing pane plays Metro's pivot motion: the headers glide so the new
//! one leads, the old pane slips away and fades, and the new pane slides in
//! from the side it came from with its lines fanning in behind it. All of
//! it is native animation on the pane, header row and line layers; panes
//! are built once per visit.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use nsh_core::prelude::*;

use crate::{
    TextStyle,
    chrome::PAGE_MARGIN,
    components::pager,
    motion::{Direction, EXP6_OUT, PANE_OUT, PANE_SHIFT, PaneSlide},
    text,
    theme::{FONT_FAMILY, LIGHT, palette, reduced_motion, size, u},
};

/// What a selection runs, with the index chosen.
pub type SelectHandler = Rc<dyn Fn(usize)>;
/// Builds a pane's content.
pub type PaneBuilder = Rc<dyn Fn() -> AnyElement>;

pub struct Pivot {
    title: Option<SharedString>,
    headers: Vec<SharedString>,
    selected: Reactive<usize>,
    content: Option<PaneBuilder>,
    panes: Vec<PaneBuilder>,
    on_select: Option<SelectHandler>,
}

/// `pivot(["all", "unread"], move || tab.get())`
pub fn pivot<S: Into<SharedString>>(headers: impl IntoIterator<Item = S>, selected: impl IntoReactive<usize>) -> Pivot {
    Pivot {
        title: None,
        headers: headers.into_iter().map(Into::into).collect(),
        selected: selected.into_reactive(),
        content: None,
        panes: Vec::new(),
        on_select: None,
    }
}

impl Pivot {
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }
    /// One content for every header (rebuilt on each change).
    pub fn child<E: IntoElement>(mut self, content: impl Fn() -> E + 'static) -> Self {
        self.content = Some(Rc::new(move || content().into_element()));
        self
    }
    /// A builder per header.
    pub fn panes<E: IntoElement, F: Fn() -> E + 'static>(mut self, panes: impl IntoIterator<Item = F>) -> Self {
        self.panes = panes.into_iter().map(|f| Rc::new(move || f().into_element()) as PaneBuilder).collect();
        self
    }
    /// Tapping a header or swiping sideways asks for that pane.
    pub fn on_select(mut self, f: impl Fn(usize) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

const HEADER_SIZE: f32 = size::HUGE * 0.86;
const HEADER_GAP: f32 = 22.;
const GLIDE: f32 = 500.;

/// How the header row glides from `prev` to `selected`: the headers to lay
/// out in order, and the row's offset at the start and end of the glide.
#[derive(Clone, Debug, PartialEq)]
pub struct HeaderGlide {
    pub order: Vec<usize>,
    pub from: f32,
    pub to: f32,
}

pub fn header_glide(widths: &[f32], prev: usize, selected: usize, direction: Direction) -> HeaderGlide {
    let len = widths.len();
    let span = |start: usize, count: usize| -> f32 { (0..count).map(|i| widths[(start + i) % len] + HEADER_GAP).sum() };
    if direction == Direction::Backward {
        let back = (prev + len - selected) % len;
        HeaderGlide { order: (0..len + back).map(|i| (selected + i) % len).collect(), from: -span(selected, back), to: 0. }
    } else {
        let ahead = (selected + len - prev) % len;
        HeaderGlide { order: (0..len + ahead).map(|i| (prev + i) % len).collect(), from: 0., to: -span(prev, ahead) }
    }
}

/// Headers in display order, the selected one first.
pub fn rotated(len: usize, selected: usize) -> Vec<usize> {
    (0..len).map(|i| (selected + i) % len.max(1)).collect()
}

/// Whether going from `from` to `to` is a forward swipe (the short way round).
pub fn is_forward(from: usize, to: usize, len: usize) -> bool {
    if len == 0 {
        return true;
    }
    let ahead = (to + len - from) % len;
    ahead != 0 && ahead <= len / 2
}

impl IntoElement for Pivot {
    fn into_element(self) -> AnyElement {
        let len = self.headers.len().max(1);
        let labels: Rc<Vec<SharedString>> = Rc::new(self.headers.iter().map(|h| SharedString::from(h.to_lowercase())).collect());
        let widths = Rc::new(RefCell::new(vec![0f32; len]));
        let builders: Rc<Vec<PaneBuilder>> = Rc::new(match self.content {
            Some(c) => vec![c; len],
            None => self.panes,
        });
        let selected = self.selected;
        let first = selected.get_untracked_or_get().min(len - 1);
        let shown = signal(first);
        let leaving = signal(None::<(usize, Direction)>);
        let order = signal(rotated(len, first));
        let row_x = animated_value(0.);
        let (out_x, out_op, in_op) = (animated_value(0.), animated_value(1.), animated_value(1.));
        let slide = PaneSlide::new();
        let last = Rc::new(Cell::new(first));

        {
            let (widths, last) = (widths.clone(), last.clone());
            effect(move || {
                let sel = selected.get().min(len - 1);
                let prev = last.replace(sel);
                if prev == sel {
                    return;
                }
                untracked(|| {
                    let direction = if is_forward(prev, sel, len) { Direction::Forward } else { Direction::Backward };
                    if reduced_motion() || len < 2 {
                        order.set(rotated(len, sel));
                        shown.set(sel);
                        return;
                    }
                    let g = header_glide(&widths.borrow(), prev, sel, direction);
                    order.set(g.order);
                    row_x.set(g.from);
                    let _ = row_x.timing(g.to, GLIDE, EXP6_OUT);
                    // The old pane slips away and fades…
                    leaving.set(Some((prev, direction)));
                    out_x.set(0.);
                    out_op.set(1.);
                    let _ = out_x.timing(-direction.sign() * PANE_SHIFT * 0.5, PANE_OUT, Easing::EaseIn);
                    let _ = out_op.timing(0., PANE_OUT, Easing::Linear);
                    // …and the new one slides in once it has gone.
                    shown.set(sel);
                    in_op.set(0.);
                    slide.sign.set(direction.sign());
                    slide.progress.set(1.);
                    let _ = Animation::sequence([
                        Animation::delay(PANE_OUT),
                        Animation::timing(&in_op, 1., Timing::new(1., Easing::Linear)),
                        Animation::timing(&slide.progress, 0., Timing::new(crate::motion::PANE_IN, EXP6_OUT)),
                    ])
                    .start();
                    set_timeout(std::time::Duration::from_millis(PANE_OUT as u64 + 20), move || leaving.set(None));
                });
            });
        }

        let on_select = self.on_select.clone();
        let header_row = {
            let (labels, widths) = (labels.clone(), widths.clone());
            move || {
                let sel = shown.get();
                view()
                    .flex_row()
                    .gap(u(HEADER_GAP))
                    .bind_translate_x(&row_x)
                    .children(order.get().into_iter().map(|ix| {
                        let widths = widths.clone();
                        let on_select = on_select.clone();
                        let is_sel = ix == sel;
                        let h = view()
                            .flex_shrink_0()
                            .on_layout(move |b: &Bounds<Points>| widths.borrow_mut()[ix] = f32::from(b.size.width))
                            .child(
                                nsh_core::prelude::text(labels[ix].clone())
                                    .font_family(FONT_FAMILY)
                                    .text_size(u(HEADER_SIZE))
                                    .font_weight(LIGHT)
                                    .number_of_lines(1u32)
                                    .text_color(move || if is_sel { palette().foreground } else { palette().subtle }),
                            );
                        match on_select.filter(|_| !is_sel) {
                            Some(f) => h.cursor_pointer().on_press(move || f(ix)).into_element(),
                            None => h.into_element(),
                        }
                    }))
                    .into_element()
            }
        };

        let pane = {
            let builders = builders.clone();
            move || {
                provide_context(slide);
                let build = builders.get(shown.get()).cloned();
                view()
                    .size_full()
                    .bind_translate_x(&slide.offset())
                    .bind_opacity(&in_op)
                    .children(build.map(|b| b()))
                    .into_element()
            }
        };
        let leaving_pane = move || {
            let Some((ix, _)) = leaving.get() else { return view().into_element() };
            let build = builders.get(ix).cloned();
            view()
                .absolute()
                .inset_0()
                .bind_translate_x(&out_x)
                .bind_opacity(&out_op)
                .children(build.map(|b| b()))
                .into_element()
        };

        let root = view()
            .relative()
            .size_full()
            .overflow_hidden()
            .child(
                view()
                    .flex_shrink_0()
                    .pt(u(12.))
                    .px(u(PAGE_MARGIN))
                    .children(self.title.map(|t| text(TextStyle::AppTitle, t)))
                    .child(view().ml(u(-2.)).overflow_hidden().child(dynamic(header_row))),
            )
            .child(
                view()
                    .flex_1()
                    .min_h(px(0.))
                    .relative()
                    .pt(u(8.))
                    .child(dynamic(leaving_pane))
                    .child(dynamic(pane)),
            );
        match self.on_select.filter(|_| len > 1) {
            Some(f) => pager::pageable(
                root,
                Rc::new(move |step| {
                    let sel = last.get();
                    f(if step > 0 { (sel + 1) % len } else { (sel + len - 1) % len });
                }),
            )
            .into_element(),
            None => root.into_element(),
        }
    }
}

/// A reactive's value without subscribing the caller.
trait Untracked<T> {
    fn get_untracked_or_get(&self) -> T;
}

impl<T: Clone + 'static> Untracked<T> for Reactive<T> {
    fn get_untracked_or_get(&self) -> T {
        untracked(|| self.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_header_leads() {
        assert_eq!(rotated(4, 0), vec![0, 1, 2, 3]);
        assert_eq!(rotated(4, 2), vec![2, 3, 0, 1]);
    }

    #[test]
    fn headers_glide_so_the_new_one_ends_up_leading() {
        let w = [100., 80., 120., 60.];
        let g = header_glide(&w, 0, 2, Direction::Forward);
        assert_eq!(g.order[2], 2);
        assert_eq!((g.from, g.to), (0., -(100. + 22. + 80. + 22.)));
        let b = header_glide(&w, 0, 3, Direction::Backward);
        assert_eq!(b.order[0], 3);
        assert_eq!((b.from, b.to), (-(60. + 22.), 0.));
    }

    #[test]
    fn direction_wraps_the_short_way() {
        assert!(is_forward(0, 1, 4));
        assert!(!is_forward(1, 0, 4));
        assert!(is_forward(3, 0, 4));
        assert!(!is_forward(0, 3, 4));
    }
}
