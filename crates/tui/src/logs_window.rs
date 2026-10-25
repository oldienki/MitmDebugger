//! Logs modal: a read-only, scrollable text view over the `logs` ring
//! buffer. Opened from `File -> Logs` or `Alt+L`; Esc or the window close
//! marker closes it. The view snapshots the ring at open, so records
//! emitted while the modal runs do not appear until reopen.

use i18n::i18n;
use logs::{LogRecord, RingHandle};
use tv::Program;
use tv::event::{Event, Key, MouseWheel};
use tv::theme::Role;
use tv::view::{Context, DrawCtx, Rect, View, ViewId};
use tv::widgets::Scroller;

/// Render ring-buffer records as display lines, oldest first.
pub fn format_records(records: &[LogRecord]) -> Vec<String> {
    records
        .iter()
        .map(|r| format!("{} [{}] {}: {}", r.ts, r.level, r.target, r.message))
        .collect()
}

/// Clamp a requested vertical scroll offset into `[0, max_scroll]`.
fn clamp_scroll(requested: i32, max_scroll: i32) -> i32 {
    requested.clamp(0, max_scroll.max(0))
}

/// A read-only scrollable view over pre-formatted log lines.
///
/// Wraps a [`Scroller`]: `delta.y` is the first visible line index,
/// `limit` is `(max_line_width, line_count)`. Keyboard (arrows, PgUp/PgDn,
/// Home/End) and the mouse wheel scroll; linked scroll bars report the
/// position. The view is selectable so it can take focus inside the modal.
pub struct LogView {
    scroller: Scroller,
    lines: Vec<String>,
    /// Logical content width: longest line, in columns.
    width: i32,
    /// `false` until the first event publishes the scroll limit and jumps
    /// to the end. [`Context`]-requiring setup cannot run in `new`.
    initialized: bool,
}

impl LogView {
    /// Create the view over `bounds` with `lines` pre-formatted.
    /// `v_scroll_bar`/`h_scroll_bar` are the [`ViewId`]s of sibling
    /// scroll bars minted by the owning window, if any.
    pub fn new(
        bounds: Rect,
        lines: Vec<String>,
        h_scroll_bar: Option<ViewId>,
        v_scroll_bar: Option<ViewId>,
    ) -> Self {
        let width = lines
            .iter()
            .map(|l| l.chars().count() as i32)
            .max()
            .unwrap_or(0);
        LogView {
            scroller: Scroller::new(bounds, h_scroll_bar, v_scroll_bar),
            lines,
            width,
            initialized: false,
        }
    }

    /// Publish the content extent to the scroll bars and scroll to the
    /// end (newest records visible). Runs lazily on the first event,
    /// which is the first point a [`Context`] is available.
    fn ensure_init(&mut self, ctx: &mut Context) {
        if self.initialized {
            return;
        }
        self.initialized = true;
        self.scroller
            .set_limit(self.width, self.lines.len() as i32, ctx);
        self.scroll_to_end(ctx);
    }

    /// Scroll so the last line is the bottom visible row.
    pub fn scroll_to_end(&mut self, ctx: &mut Context) {
        let max = (self.lines.len() as i32 - self.scroller.state().size.y).max(0);
        self.scroller.scroll_to(self.scroller.delta.x, max, ctx);
    }

    /// Maximum legal vertical offset: `lines - view_height`, floored at 0.
    fn max_scroll(&self) -> i32 {
        (self.lines.len() as i32 - self.scroller.state().size.y).max(0)
    }

    /// Request a new vertical offset, clamped to the legal range.
    fn scroll_by(&mut self, dy: i32, ctx: &mut Context) {
        let target = clamp_scroll(self.scroller.delta.y + dy, self.max_scroll());
        self.scroller.scroll_to(self.scroller.delta.x, target, ctx);
    }

    fn scroll_to_absolute(&mut self, y: i32, ctx: &mut Context) {
        let target = clamp_scroll(y, self.max_scroll());
        self.scroller.scroll_to(self.scroller.delta.x, target, ctx);
    }
}

#[tv::delegate(to = scroller)]
impl View for LogView {
    /// Draw the visible slice: screen row `y` shows line `delta.y + y`,
    /// column-clipped by `delta.x`. Remaining rows are filled with the
    /// scroller background.
    fn draw(&mut self, ctx: &mut DrawCtx) {
        let style = ctx.style(Role::ScrollerNormal);
        let size = self.scroller.state().size;
        let extent = self.scroller.state().get_extent();
        ctx.fill(extent, ' ', style);

        let first = self.scroller.delta.y as usize;
        let x_off = self.scroller.delta.x;
        for y in 0..size.y {
            let Some(line) = self.lines.get(first + y as usize) else {
                break;
            };
            let visible: String = line
                .chars()
                .skip(x_off.max(0) as usize)
                .take(size.x.max(0) as usize)
                .collect();
            if !visible.is_empty() {
                ctx.put_str(0, y, &visible, style);
            }
        }
    }

    /// Scroll keys + wheel; scrollbar broadcasts fall through to the
    /// embedded [`Scroller`]'s own handler.
    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Context) {
        self.ensure_init(ctx);
        let page = self.scroller.state().size.y.max(1);
        match ev {
            Event::KeyDown(ke) => {
                match ke.key {
                    Key::Up => self.scroll_by(-1, ctx),
                    Key::Down => self.scroll_by(1, ctx),
                    Key::Left => self.scroller.scroll_to(
                        self.scroller.delta.x - 1,
                        self.scroller.delta.y,
                        ctx,
                    ),
                    Key::Right => self.scroller.scroll_to(
                        self.scroller.delta.x + 1,
                        self.scroller.delta.y,
                        ctx,
                    ),
                    Key::PageUp => self.scroll_by(-page, ctx),
                    Key::PageDown => self.scroll_by(page, ctx),
                    Key::Home => self.scroll_to_absolute(0, ctx),
                    Key::End => self.scroll_to_end(ctx),
                    _ => {
                        self.scroller.handle_event(ev, ctx);
                        return;
                    }
                }
                ev.clear();
            }
            Event::MouseWheel(me) => {
                match me.wheel {
                    MouseWheel::Up => self.scroll_by(-3, ctx),
                    MouseWheel::Down => self.scroll_by(3, ctx),
                    _ => {
                        self.scroller.handle_event(ev, ctx);
                        return;
                    }
                }
                ev.clear();
            }
            _ => self.scroller.handle_event(ev, ctx),
        }
    }
}

/// Build the Logs dialog over `lines`: a centered `Dialog` at 80% of the
/// desktop containing a [`LogView`] with a vertical scroll bar. A `Dialog`
/// (not a plain `Window`) because it translates Esc into `Command::CANCEL`
/// natively and ships the close marker — both required close paths.
/// Split from [`open`] so tests can exercise the modal without a
/// [`RingHandle`] (its constructor is crate-private in `logs`).
fn build_dialog(screen: Rect, lines: Vec<String>) -> tv::dialog::Dialog {
    use tv::widgets::ScrollBar;

    let w = screen.b.x - screen.a.x;
    let h = screen.b.y - screen.a.y;
    let width = (w * 4 / 5).max(20);
    let height = (h * 4 / 5).max(8);
    let origin = (screen.a.x + (w - width) / 2, screen.a.y + (h - height) / 2);

    let mut dialog = tv::dialog::Dialog::new(
        Rect::new(origin.0, origin.1, origin.0 + width, origin.1 + height),
        Some(i18n!("Logs").to_string()),
    );
    // Local coords: vertical bar on the right edge inside the frame
    // (bordered: one column inset from the corners).
    let vsb = dialog.insert_child(Box::new(ScrollBar::new(Rect::new(
        width - 2,
        1,
        width - 1,
        height - 1,
    ))));
    // The LogView fills the client area minus the scroll-bar column.
    dialog.insert_child(Box::new(LogView::new(
        Rect::new(1, 1, width - 2, height - 1),
        lines,
        None,
        Some(vsb),
    )));
    dialog
}

/// Open the Logs modal over `logs`: snapshots the ring buffer, then runs
/// the dialog built by [`build_dialog`] through `exec_view`. Blocks until
/// Esc or the close marker ends the modal (`Command::CANCEL`).
pub fn open(program: &mut Program, logs: &RingHandle) {
    let lines = format_records(&logs.snapshot());
    let screen = program.desktop_rect();
    program.exec_view(Box::new(build_dialog(screen, lines)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;

    fn rec(ts: &str, level: Level, target: &str, msg: &str) -> LogRecord {
        LogRecord {
            ts: ts.into(),
            level,
            target: target.into(),
            message: msg.into(),
        }
    }

    #[test]
    fn format_oldest_first() {
        let out = format_records(&[
            rec("a", Level::INFO, "t", "first"),
            rec("b", Level::WARN, "t", "last"),
        ]);
        assert_eq!(out.len(), 2);
        assert!(out[0].contains("first"));
        assert!(out[1].contains("last"));
        assert!(out[0].starts_with("a [INFO] t:"));
    }

    #[test]
    fn format_empty_is_empty() {
        assert!(format_records(&[]).is_empty());
    }

    #[test]
    fn format_unicode_preserved() {
        let out = format_records(&[rec("t", Level::INFO, "x", "привет 🌍")]);
        assert!(out[0].contains("привет 🌍"));
    }

    #[test]
    fn clamp_scroll_bounds() {
        assert_eq!(clamp_scroll(-5, 10), 0);
        assert_eq!(clamp_scroll(15, 10), 10);
        assert_eq!(clamp_scroll(5, 10), 5);
        assert_eq!(clamp_scroll(3, -2), 0);
    }

    #[test]
    fn logview_width_is_longest_line() {
        let v = LogView::new(
            Rect::new(0, 0, 10, 5),
            vec!["ab".into(), "abcde".into(), "x".into()],
            None,
            None,
        );
        assert_eq!(v.width, 5);
    }

    #[test]
    fn max_scroll_floors_at_zero() {
        // 3 lines in a 5-row view: nothing to scroll.
        let v = LogView::new(Rect::new(0, 0, 10, 5), vec!["a".into(); 3], None, None);
        assert_eq!(v.max_scroll(), 0);
        // 10 lines in a 5-row view: 5 rows of scroll.
        let v = LogView::new(Rect::new(0, 0, 10, 5), vec!["a".into(); 10], None, None);
        assert_eq!(v.max_scroll(), 5);
    }

    // -- headless modal tests ----------------------------------------------

    use tv::backend::HeadlessBackend;
    use tv::desktop::Desktop;
    use tv::theme::Theme;
    use tv::timer::ManualClock;

    /// A `Program` with a real desktop over a headless backend, mirroring
    /// `program_with_desktop` in tvision-rs's own tests.
    fn headless_program(w: u16, h: u16) -> (Program, tv::backend::HeadlessHandle) {
        let (backend, handle) = HeadlessBackend::new(w, h);
        let program = Program::new(
            Box::new(backend),
            Box::new(ManualClock::new(0)),
            Theme::default(),
            |r| {
                Some(Box::new(Desktop::new(r, |r2| {
                    Some(Desktop::init_background(r2))
                })))
            },
            |_r| None,
            |_r| None,
        );
        (program, handle)
    }

    fn many_lines(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("line {i:03}")).collect()
    }

    /// Esc ends the modal with Command::CANCEL.
    #[test]
    fn esc_closes_modal() {
        let (mut program, handle) = headless_program(80, 25);
        handle.push_key(Key::Esc, tv::event::KeyModifiers::default());
        let screen = Rect::new(0, 0, 80, 24);
        let result = program.exec_view(Box::new(build_dialog(screen, many_lines(50))));
        assert_eq!(result, tv::Command::CANCEL, "Esc ends the modal");
    }

    /// The close marker (Command::CLOSE on the modal window) ends the modal.
    #[test]
    fn close_marker_closes_modal() {
        let (mut program, _handle) = headless_program(80, 25);
        program.put_event(Event::Command(tv::Command::CLOSE));
        let screen = Rect::new(0, 0, 80, 24);
        let result = program.exec_view(Box::new(build_dialog(screen, many_lines(50))));
        assert_eq!(
            result,
            tv::Command::CANCEL,
            "CLOSE -> CANCEL ends the modal"
        );
    }

    /// The dialog opens scrolled to the end: the last line is visible.
    #[test]
    fn opens_scrolled_to_end() {
        let (mut program, handle) = headless_program(80, 25);
        handle.push_key(Key::Esc, tv::event::KeyModifiers::default());
        let screen = Rect::new(0, 0, 80, 24);
        program.exec_view(Box::new(build_dialog(screen, many_lines(50))));
        let snap = handle.snapshot();
        assert!(snap.contains("line 049"), "last line visible:\n{snap}");
        assert!(!snap.contains("line 000"), "top scrolled away:\n{snap}");
    }

    /// An empty ring still opens and closes cleanly.
    #[test]
    fn empty_ring_opens_and_closes() {
        let (mut program, handle) = headless_program(80, 25);
        handle.push_key(Key::Esc, tv::event::KeyModifiers::default());
        let screen = Rect::new(0, 0, 80, 24);
        let result = program.exec_view(Box::new(build_dialog(screen, vec![])));
        assert_eq!(result, tv::Command::CANCEL);
        assert!(handle.snapshot().contains("Logs"), "title drawn");
    }

    #[test]
    fn snapshot_isolation() {
        // Records pushed after a snapshot must not appear: this is what
        // makes the modal safe to keep open while logging continues.
        let ring = logs::RingBuffer::new(8);
        ring.push(rec("t", Level::INFO, "x", "old"));
        let lines = format_records(&ring.snapshot());
        ring.push(rec("t", Level::INFO, "x", "new"));
        assert!(lines.iter().any(|l| l.contains("old")));
        assert!(!lines.iter().any(|l| l.contains("new")));
    }
}
