use leptos::*;

#[component]
pub fn Dots(cx: Scope, is_loading: ReadSignal<bool>) -> impl IntoView {
    view! { cx, {move || {
        if is_loading.get() {
            view!{cx,
                <div class="col-3 ms-3">
                <div class="snippet" data-title="dot-falling">
                    <div class="stage">
                    <div class="dot-falling"></div>
                    </div>
                </div>
                </div>
            }.into_view(cx)
        } else {
            view!{cx, <></>}.into_view(cx)
        }
    }}}
}
