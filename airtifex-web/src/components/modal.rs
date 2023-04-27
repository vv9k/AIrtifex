use leptos::*;

#[component]
pub fn Modal(
    cx: Scope,
    modal_id: &'static str,
    title: ReadSignal<String>,
    body: Signal<View>,
    footer: ReadSignal<View>,
) -> impl IntoView {
    view!{ cx,
    <div class="modal" id={modal_id} tabindex="-1">
      <div class="modal-dialog">
        <div class="modal-content bg-darker text-white">
          <div class="modal-header">
            <h5 class="modal-title">{title}</h5>
            <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
          </div>
          <div class="modal-body">
            {body}
          </div>
          <div class="modal-footer">
            {footer}
          </div>
        </div>
      </div>
    </div>}.into_view(cx)
}

#[component]
pub fn RemoveModal<F>(
    cx: Scope,
    modal_id: &'static str,
    target: &'static str,
    entry: ReadSignal<Option<String>>,
    remove_action_fn: F,
) -> impl IntoView
where
    F: FnOnce() + Copy + 'static,
{
    let footer = create_rw_signal(
        cx,
        view! {cx,
                <button
                  type="button"
                  data-bs-dismiss="modal"
                  on:click=move |_| remove_action_fn()
                  class="btn btn-danger"
                >
                  "Remove"
                </button>
                <button
                  type="button"
                  class="btn btn-secondary"
                  data-bs-dismiss="modal"
                >
                  "Cancel"
                </button>
        }
        .into_view(cx),
    );
    let title = create_rw_signal(cx, format!("Remove {target}"));
    let body = Signal::derive(cx, move || {
        view! {cx,
            <p>{
                format!(
                    "Are you sure you want to remove {target} {}?",
                    entry.get().unwrap_or_default()
                )
            }</p>
        }
        .into_view(cx)
    });

    view! {cx,
        <Modal
            modal_id
            title=title.read_only()
            body=body
            footer=footer.read_only()
    />}
    .into_view(cx)
}
