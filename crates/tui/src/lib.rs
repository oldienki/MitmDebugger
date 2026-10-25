use backend::Backend;
mod app;
mod commands;
mod desktop;
mod logs_window;

pub fn run(logs: logs::RingHandle, backend: Backend) -> anyhow::Result<()> {
    let mut app = app::App::new(backend, logs)?;
    app.run()
}
