use leptos::*;
use leptos_scholar::*;

fn main() {
    mount_to_body(|| view! { <Render /> })
}

#[component]
fn render() -> impl IntoView {
    let (author_id, set_author_id) = create_signal("H7sOPf8AAAAJ".to_string());

    // our resource
    let async_data = create_resource(author_id, |author_id| async move {
        fetch_info(author_id)
            .await
            .unwrap_or_else(|e| e.to_string())
    });

    view! {
        <main>
            <h1>Scholar</h1>
                <p>
                Extract the citation info from Google scholar
                </p>

            <label>Author id:</label>
            <input type="text"
            on:input=move |ev| {set_author_id(event_target_value(&ev));}
            prop:value=author_id
            />

            <pre>
            <Suspense
                fallback=move || view! { <p>" Loading "</p> }
            >
            {async_data.get()}

            </Suspense>
            </pre>
        </main>
    }
}
