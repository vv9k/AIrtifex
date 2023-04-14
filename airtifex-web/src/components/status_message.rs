use leptos::*;

#[derive(Clone)]
pub enum Message {
    Success(String),
    Error(String),
    Empty,
}

#[component]
pub fn StatusMessage(cx: Scope, message: RwSignal<Message>) -> impl IntoView {
    view! { cx,
        {move || {
            let view = match message.get() {
                Message::Success(msg) => {
                      view!{ cx, <p class="text-airtifex-green text-md-center mt-2">{msg}</p> }.into_view(cx)

                }
                Message::Error(msg) => {
                      view!{ cx, <p class="text-airtifex-red text-md-center mt-2">{msg}</p> }.into_view(cx)
                }
                Message::Empty => {
                      view!{ cx, <p></p> }.into_view(cx)
                }
            };

            message.update(|m| *m = Message::Empty);
            view
       }}
    }
    .into_view(cx)
}
