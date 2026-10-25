use backend::Backend;
use tv::Program;
use tv::backend::CrosstermBackend;
use tv::theme::Theme;
use tv::timer::SystemClock;

pub struct App {
    #[allow(unused)]
    backend: Backend,
    #[allow(unused)]
    logs: logs::RingHandle,
    this: Program,
}

impl App {
    pub fn new(backend: Backend, logs: logs::RingHandle) -> anyhow::Result<Self> {
        let this = Program::new(
            Box::new(CrosstermBackend::new()?),
            Box::new(SystemClock::new()),
            Theme::default(),
            |rect| Some(crate::desktop::make_desktop(rect)),
            |rect| Some(crate::desktop::make_status_line(rect)),
            |rect| Some(crate::desktop::make_menu_bar(rect)),
        );
        Ok(Self {
            backend,
            logs,
            this,
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let logs = self.logs.clone();
        self.this.run_app(move |prog, cmd| {
            if cmd == crate::commands::CMD_LOGS {
                crate::logs_window::open(prog, &logs);
            }
        });
        Ok(())
    }
}
