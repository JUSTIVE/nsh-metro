//! The catalog's table of contents: every page, its sidebar group, the
//! paragraph shown above the demo, and the usage snippet beside it.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Foundations,
    Tiles,
    Navigation,
    Controls,
    Progress,
    Lists,
    Overlays,
    Chrome,
}

impl Group {
    pub const ALL: [Group; 8] = [
        Group::Foundations,
        Group::Tiles,
        Group::Navigation,
        Group::Controls,
        Group::Progress,
        Group::Lists,
        Group::Overlays,
        Group::Chrome,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Group::Foundations => "foundations",
            Group::Tiles => "tiles",
            Group::Navigation => "navigation",
            Group::Controls => "controls",
            Group::Progress => "progress",
            Group::Lists => "lists",
            Group::Overlays => "overlays",
            Group::Chrome => "chrome",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Palette,
    Typography,
    Icons,
    Tile,
    TileGrid,
    Turnstile,
    Pivot,
    Panorama,
    AppBar,
    Button,
    RoundButton,
    Hyperlink,
    ToggleSwitch,
    CheckBox,
    RadioButton,
    Slider,
    ListPicker,
    TextBox,
    ProgressBar,
    ProgressDots,
    ProgressRing,
    ListItem,
    JumpList,
    MessageBox,
    ContextMenu,
    Toast,
    SettingsFlyout,
    SystemTray,
    Header,
    Motion,
}

impl Page {
    pub const ALL: [Page; 30] = [
        Page::Palette,
        Page::Typography,
        Page::Icons,
        Page::Tile,
        Page::TileGrid,
        Page::Turnstile,
        Page::Pivot,
        Page::Panorama,
        Page::AppBar,
        Page::Button,
        Page::RoundButton,
        Page::Hyperlink,
        Page::ToggleSwitch,
        Page::CheckBox,
        Page::RadioButton,
        Page::Slider,
        Page::ListPicker,
        Page::TextBox,
        Page::ProgressBar,
        Page::ProgressDots,
        Page::ProgressRing,
        Page::ListItem,
        Page::JumpList,
        Page::MessageBox,
        Page::ContextMenu,
        Page::Toast,
        Page::SettingsFlyout,
        Page::SystemTray,
        Page::Header,
        Page::Motion,
    ];

    /// Looks a page up by name, ignoring case, spaces, dashes and underscores
    /// (`CATALOG_PAGE=tile-grid`, `appbar`, `List Picker` all work).
    pub fn find(name: &str) -> Option<Page> {
        let key = |s: &str| s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
        let wanted = key(name);
        Page::ALL.into_iter().find(|p| key(p.name()) == wanted)
    }

    pub fn group(self) -> Group {
        match self {
            Page::Palette | Page::Typography | Page::Icons => Group::Foundations,
            Page::Tile | Page::TileGrid | Page::Turnstile => Group::Tiles,
            Page::Pivot | Page::Panorama | Page::AppBar => Group::Navigation,
            Page::Button
            | Page::RoundButton
            | Page::Hyperlink
            | Page::ToggleSwitch
            | Page::CheckBox
            | Page::RadioButton
            | Page::Slider
            | Page::ListPicker
            | Page::TextBox => Group::Controls,
            Page::ProgressBar | Page::ProgressDots | Page::ProgressRing => Group::Progress,
            Page::ListItem | Page::JumpList => Group::Lists,
            Page::MessageBox | Page::ContextMenu | Page::Toast | Page::SettingsFlyout => Group::Overlays,
            Page::SystemTray | Page::Header | Page::Motion => Group::Chrome,
        }
    }

    /// Lower-case display name, as Metro would set it.
    pub fn name(self) -> &'static str {
        match self {
            Page::Palette => "accents",
            Page::Typography => "typography",
            Page::Icons => "icons",
            Page::Tile => "tile",
            Page::TileGrid => "tile grid",
            Page::Turnstile => "turnstile",
            Page::Pivot => "pivot",
            Page::Panorama => "panorama",
            Page::AppBar => "app bar",
            Page::Button => "button",
            Page::RoundButton => "round button",
            Page::Hyperlink => "hyperlink",
            Page::ToggleSwitch => "toggle switch",
            Page::CheckBox => "check box",
            Page::RadioButton => "radio button",
            Page::Slider => "slider",
            Page::ListPicker => "list picker",
            Page::TextBox => "text box",
            Page::ProgressBar => "progress bar",
            Page::ProgressDots => "progress dots",
            Page::ProgressRing => "progress ring",
            Page::ListItem => "list item",
            Page::JumpList => "jump list",
            Page::MessageBox => "message box",
            Page::ContextMenu => "context menu",
            Page::Toast => "toast",
            Page::SettingsFlyout => "settings flyout",
            Page::SystemTray => "system tray",
            Page::Header => "page header",
            Page::Motion => "motion",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Page::Palette => {
                "Windows Phone let the owner choose exactly two things: a dark or light ground and one of twenty accents. Every component reads them from the global Theme, so an app never hard-codes a colour. Tap a swatch to re-skin the whole catalog, then compare the resolved palette tokens below it."
            }
            Page::Typography => {
                "Type is the interface. The Metro ramp runs from 56 px light page titles down to 14 px captions, set in the platform's closest relative of Segoe WP. Each TextStyle carries its size, weight and colour role, and composes as an ordinary styled div."
            }
            Page::Icons => {
                "Single-weight geometric glyphs with square caps, embedded in the binary as SVG markup and drawn as vectors by the platform. They tint with the element's text colour, so the same glyph works on the ground, on an accent tile and inside a round button."
            }
            Page::Tile => {
                "The start-screen tile: a flat accent square with its name in the corner and an optional count. Four footprints (small, medium, wide, large) plus custom front faces and live back faces that periodically slide up, the WP hub-tile peek. Tap the mail tile to bump its count."
            }
            Page::Turnstile => {
                "The start screen's turnstile, on its own. When the grid appears its tiles swing open about the screen's left edge, top-left first; tap a tile and they swing away in a wave, the tapped one last, then (here) straight back in. Each tile really rotates about the hinge, in perspective. Replay the entrance on its own."
            }
            Page::TileGrid => {
                "TileGrid packs tiles the way the start screen did: 2×2 blocks, medium and larger tiles aligned to block corners, small tiles filling quadrants first-fit. Flip the switch for the six-column \"show more tiles\" layout; the same tiles re-pack at a smaller unit. With turnstile on, tapping people or outlook swings the tiles out (the tapped one last) and then opens a panorama or a pivot."
            }
            Page::Pivot => {
                "A pivot shows one pane under a row of huge lower-case headers. The selected header always leads and the rest wrap around it, dimmed. Tap a header, Shift + scroll the wheel, or swipe on a trackpad; the old pane slips away and the new one slides in from the direction you moved, its lines fanning in one after another (sender, then subject, then preview)."
            }
            Page::Panorama => {
                "The hub: one long canvas behind a title far wider than the screen. Sections snap one at a time with the next one peeking in from the right, and the title drifts slower than the content for parallax. It wraps around: past the last section comes the first, while the title runs back to its start. Opening the hub, the sections fly in from the right with the title behind them. Tap the peeking section, Shift + scroll the wheel, or swipe to move."
            }
            Page::AppBar => {
                "Up to four round glyph buttons along the bottom edge and the ··· that expands the bar to reveal labels and lower-case menu items. Minimized mode shows only the ··· strip until opened. Expansion state belongs to the bar; actions collapse it."
            }
            Page::Button => {
                "The Metro button is a hard 3 px outline with no fill; pressing floods it with the accent. Accent buttons are filled at rest to mark the default action. Buttons can carry a glyph, be disabled, or stretch full width."
            }
            Page::RoundButton => {
                "The app-bar glyph in a 2 px ring, also usable standalone. A filled round button knocks the glyph out of a solid disc to show a toggled-on state; the diameter is configurable."
            }
            Page::Hyperlink => {
                "An inline, underlined text link that takes the accent while pressed. Use it for secondary navigation such as privacy statements or \"learn more\"."
            }
            Page::ToggleSwitch => {
                "The WP8 toggle switch: a subtle header, the current On/Off word in large type and the rectangular track whose thumb slides with the accent trailing it. Labels can be customised and the switch disabled."
            }
            Page::CheckBox => {
                "A square check box with a label. Controlled: the caller owns the value and receives the new one in on_change, so the catalog keeps these in its own state."
            }
            Page::RadioButton => {
                "Round radio buttons. Group them yourself: the caller decides which one is selected and each button reports when it is chosen."
            }
            Page::Slider => {
                "A thick track with an accent fill and a thin upright thumb. Click or drag anywhere on it; the drag keeps tracking outside the bounds. Optional step snapping, header and disabled state."
            }
            Page::ListPicker => {
                "The toolkit ListPicker shows the current choice in a field and expands the choices in place when tapped. The second picker here drives the global accent."
            }
            Page::TextBox => {
                "A translucent field that turns solid white with black ink while focused, framed by an accent border. The field is the platform's own text field: word motion, selection, undo, clipboard and IME for free. It edits a Signal<String>; Enter arrives in on_submit."
            }
            Page::ProgressBar => {
                "The determinate progress bar is a flat 4 px accent strip over a tinted track. Drag the slider to drive it."
            }
            Page::ProgressDots => {
                "The indeterminate bar: five accent dots rush in, crowd and slow through the middle, then rush out. Usually shown at the very top of the page while something loads."
            }
            Page::ProgressRing => {
                "Dots chasing each other round a circle, bunching at the top on each lap. The diameter scales the dots with it."
            }
            Page::ListItem => {
                "The two-line Metro list item: a large light title over an accent or subtle subtitle, an optional third line, a leading square (glyph, letters or any element) and a trailing element. Tap the last group to select a row."
            }
            Page::JumpList => {
                "The LongListSelector: groups under accent letter squares. Tap a letter to open the jump grid; letters with no contacts are greyed out, the rest scroll the list to their group."
            }
            Page::ContextMenu => {
                "The WP Toolkit's context menu: right-click a row (or press and hold it) and a chrome panel of lower-case commands swivels open at the pointer over a lightly dimmed page. A command closes the menu and runs; a click outside closes it. Near the window's right or bottom edge it opens to the left or above."
            }
            Page::MessageBox => {
                "A full-width chrome band pinned to the top of the screen over a dimmed page. It blocks the page beneath, can carry extra content such as a check box, and offers up to two buttons. Tap the dimmed page to dismiss."
            }
            Page::Toast => {
                "A toast is an accent strip that drops in from the top edge with an icon, a bold title and a one-line message. This demo dismisses it after three seconds using an nsh timer; tapping it dismisses it early."
            }
            Page::SettingsFlyout => {
                "Windows 8's settings flyout: a full-height panel that slides in from the right edge under an accent header with a back button and a title. The page isn't dimmed; tap anywhere outside the panel to dismiss it (light dismiss). Narrow is 346 px, wide 646 px. Build it once, bound to whether it is open: it slides itself back out when that turns false."
            }
            Page::SystemTray => {
                "The status strip along the top edge: signal, wi-fi, battery and clock. It is transparent by default so the page shows through, or opaque in the chrome colour. The phone frame's own tray follows the controls here."
            }
            Page::Header => {
                "The two-line page header: the application name in small upper-case semibold over a huge lower-case page title, inset by the standard 24 px page margin."
            }
            Page::Motion => {
                "Metro's own motions, after the Windows Phone Toolkit, each replaying on its own: turnstile for pages, turnstile feather for lists, swivel for pop-ups, slide for pivot panes, and the tilt effect under the pointer. On nsh the turns are real 3D rotations in perspective, run by Core Animation: a page turns about its hinge and narrows as it swings away."
            }
        }
    }

    pub fn snippet(self) -> &'static str {
        match self {
            Page::Palette => {
                r#"// re-skin the surface
Theme::set_mode(Mode::Light);
Theme::set_accent(Accent::Lime);

// bind colours: a theme change updates only these
view()
    .bg(move || palette().background)
    .text_color(move || palette().foreground)"#
            }
            Page::Typography => {
                r#"text(TextStyle::Title1, "settings").lowercase()
text(TextStyle::AppTitle, "outlook") // upper-cased
text(TextStyle::Subtle, "last synced 2 min ago")

// content may be a closure
text(TextStyle::Large, move || format!("{n} strains"))"#
            }
            Page::Icons => {
                r#"// SVG markup embedded in the binary,
// tinted through currentColor
Icon::Mail.render(32., move || palette().foreground)
icon(Icon::Search, 20., move || palette().accent)

for i in Icon::ALL { /* i.name(), i.markup() */ }"#
            }
            Page::Tile => {
                r#"tile("mail", TileSize::Medium)
    .title("mail")
    .icon(Icon::Mail)
    .count(unread)
    .on_click(|| { /* open */ })

tile("people", TileSize::Wide)
    .title("people")
    .front(|_size| view().child(..))
    .back_text("ben", ["new photo"])
    .period(11.)
    .phase(0.5)"#
            }
            Page::Turnstile => {
                r#"tile_grid()
    .turnstile(true)       // swing in on appear, out on launch
    .replay(replays.read_only()) // bump to replay
    .tile(tile("mail", TileSize::Medium)
        // runs after the exit wave has finished
        .on_click(|| launch_mail()))"#
            }
            Page::TileGrid => {
                r#"tile_grid()
    .turnstile(true) // tiles swing out, then on_click runs
    .leave(leaving.read_only())
    .columns(if more { 6 } else { 4 })
    .metrics(unit, gap)
    .tile(tile("people", TileSize::Wide)
        .on_click(move || app.set(Some(People))))"#
            }
            Page::Pivot => {
                r#"let ix = signal(0);
pivot(["inbox", "unread", "flagged"], ix)
    .title("outlook")
    .on_select(move |i| ix.set(i))
    .panes(FOLDERS.map(|f| move || pane_for(f)))"#
            }
            Page::Panorama => {
                r#"panorama("music+videos", ix)
    .section_width(432.)
    .on_select(move |i| ix.set(i))
    .section("collection", collection)
    .section("history", history)
    .section("new", new_items)"#
            }
            Page::AppBar => {
                r#"app_bar()
    .mode(AppBarMode::Minimized)
    .button(Icon::Add, "new", || ..)
    .button(Icon::Sync, "sync", || ..)
    .disabled_button(Icon::Delete, "delete")
    .menu_item("settings", || ..)
    .opacity(0.9)"#
            }
            Page::Button => {
                r#"button("send").on_click(|| ..)
button("save").accent()
button("delete").disabled(true)
button("share").icon(Icon::Share)
button("sign in").full_width()"#
            }
            Page::RoundButton => {
                r#"round_button(Icon::Play)
    .diameter(72.)
    .on_click(|| ..)

round_button(Icon::Favorite)
    .filled(favorite)"#
            }
            Page::Hyperlink => {
                r#"hyperlink("privacy statement")
    .on_click(|| { /* navigate */ })"#
            }
            Page::ToggleSwitch => {
                r#"let wifi = signal(true);
toggle_switch(wifi)
    .header("wi-fi networking")
    .on_change(move |on| wifi.set(on))

toggle_switch(v)
    .labels("Visible", "Hidden")
    .disabled(true)"#
            }
            Page::CheckBox => {
                r#"check_box(sync)
    .label("sync contacts")
    .on_change(move |c| sync.set(c))"#
            }
            Page::RadioButton => {
                r#"for (ix, label) in ROUTES.iter().enumerate() {
    radio_button(move || route.get() == ix)
        .label(*label)
        .on_select(move || route.set(ix))
}"#
            }
            Page::Slider => {
                r#"slider(volume) // a Signal<f32>, 0–1
    .header("ringer volume")
    .on_change(move |v| volume.set(v))

slider(brightness).step(0.1)
slider(0.3).disabled(true)"#
            }
            Page::ListPicker => {
                r#"list_picker(RINGTONES, ringtone)
    .header("ringtone")
    .on_select(move |ix| ringtone.set(ix))"#
            }
            Page::TextBox => {
                r#"let name = signal(String::new());
text_box(name)
    .header("name")
    .placeholder("your name")
    .on_submit(|text| ..)

text(TextStyle::Large, move || name.get())"#
            }
            Page::ProgressBar => r#"progress_bar(move || done.get())"#,
            Page::ProgressDots => {
                r#"// indeterminate: the five flowing dots
when(move || loading.get(), progress_dots)"#
            }
            Page::ProgressRing => {
                r#"progress_ring()
progress_ring().diameter(96.)"#
            }
            Page::ListItem => {
                r#"list_item("ben thompson")
    .subtitle("mobile")          // accent
    .detail("seattle")           // third line
    .leading(Leading::Glyph("bt".into()))
    .leading_color(Accent::Teal.color())
    .trailing(Icon::ChevronRight.render(..))
    .on_click(|| ..)

list_item("plain").subtle_subtitle()"#
            }
            Page::JumpList => {
                r#"jump_list()
    .keys(ALPHABET) // empty letters grey out
    .groups(by_letter.map(|(key, rows)| {
        ListGroup::new(key, rows)
    }))"#
            }
            Page::ContextMenu => {
                r#"with_context_menu(
    list_item("kristin").subtitle("mobile"),
    context_menu()
        .item("pin to start", || ..)
        .item("edit", || ..)
        .disabled_item("delete")  // greyed out
)"#
            }
            Page::MessageBox => {
                r#"// last child of a relative() root
when(move || asking.get(), move || {
    message_box("delete strain?")
        .message("it will be removed.")
        .child(check_box(..))
        .button("delete", ..)
        .button("cancel", ..)
        .on_dismiss(..)
})"#
            }
            Page::Toast => {
                r#"effect(move || {
    if toast.get().is_some() {
        set_timeout(secs(3), move || toast.set(None));
    }
});

toast("Messaging")
    .icon(Icon::Mail)
    .message("are we still on?")"#
            }
            Page::SettingsFlyout => {
                r#"// last child of a relative() root, once
settings_flyout(open) // slides itself in and out
    .title("options")
    .width(FlyoutWidth::Narrow) // or Wide
    .child(move || toggle_switch(sync).header("sync"))
    .on_back(..)    // defaults to on_dismiss
    .on_dismiss(move || open.set(false))"#
            }
            Page::SystemTray => {
                r#"system_tray("12:38")
    .battery(0.8)
    .opaque()"#
            }
            Page::Header => {
                r#"page_header("wi-fi")
    .app_title("settings")"#
            }
            Page::Motion => {
                r#"use nsh_metro::motion::{self, Direction};

// a real rotation about a hinge, on Core Animation
motion::turnstile_in(Direction::Forward, delay, hinge, row)
motion::swivel_in(dialog)
motion::tilt(tile)

let slide = PaneSlide::new();
provide_context(slide); // list items trail it
slide.play(Direction::Forward);"#
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_is_findable_by_name() {
        for p in Page::ALL {
            assert_eq!(Page::find(p.name()), Some(p));
        }
        assert_eq!(Page::find("tile-grid"), Some(Page::TileGrid));
        assert_eq!(Page::find("AppBar"), Some(Page::AppBar));
        assert_eq!(Page::find("nope"), None);
    }
}
