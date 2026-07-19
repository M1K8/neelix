use anyhow::Result;

// Use gpui & gpui-component to present a modal dialog; this will run a transient
// gpui::Application to display a single dialog window, then exit when closed.
#[cfg(not(test))]
pub fn show_error_dialog(title: &str, message: &str) -> Result<()> {
    use gpui::Application;
    use gpui::WindowOptions;
    use gpui::{AppContext, Context, IntoElement, Render};
    use gpui_component::{Root, init};

    struct ErrorDialog {
        title: String,
        message: String,
    }

    impl Render for ErrorDialog {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            use gpui::div;
            use gpui::prelude::*;
            use gpui_component::button::Button as GpButton;
            use gpui_component::button::ButtonVariants;

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .size_full()
                .child(self.title.clone())
                .child(self.message.clone())
                .child(
                    GpButton::new("ok")
                        .primary()
                        .label("OK")
                        .on_click(|_ev, _window, _cx| {
                            // This dialog is only shown for fatal errors
                            std::process::exit(1);
                        }),
                )
        }
    }

    let title_s = title.to_string();
    let message_s = message.to_string();
    let app = Application::new();
    app.run(move |cx| {
        init(cx);
        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let _ = window.set_window_title(&title_s);
                let view = cx.new(|_| ErrorDialog {
                    title: title_s.clone(),
                    message: message_s.clone(),
                });
                cx.new(|cx| Root::new(view, window, cx))
            })?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });

    Ok(())
}

// Stubbed version for tests so showing a native dialog doesn't kill or block the test harness
#[cfg(test)]
pub fn show_error_dialog(_title: &str, _message: &str) -> Result<()> {
    Ok(())
}
