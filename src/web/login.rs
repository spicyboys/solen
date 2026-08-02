use topcoat::{
    Result,
    context::{Cx, CxBuilder},
    router::{Body, Next, Response, error::redirect, layer, page},
    view::view,
};

use crate::{
    components::button::{ButtonSize, ButtonVariant, button_variants},
    web::auth,
};

#[layer]
async fn loggedin(cx: &mut CxBuilder, body: Body, next: Next<'_>) -> Result<Response> {
    if auth::current_user_id(cx).await.is_some() {
        Err(redirect("/").into())
    } else {
        let response = next.run(cx, body).await?;
        Ok(response)
    }
}

#[page]
pub(crate) async fn login(cx: &Cx) -> Result {
    if auth::current_user_id(cx).await.is_some() {
        return Err(redirect("/").into());
    }
    view! {
        <main class="flex min-h-dvh items-center justify-center p-4">
            <div
                class="flex flex-col items-center gap-4 rounded-lg border border-border bg-background p-8 text-center shadow-sm"
            >
                <div class="space-y-1">
                    <h1 class="text-2xl font-semibold">"Solen"</h1>
                </div>
                <a
                    href="/oauth/discord"
                    class=(button_variants(
                        ButtonVariant::Primary,
                        ButtonSize::Md,
                    ))
                >
                    "Log in with Discord"
                </a>
            </div>
        </main>
    }
}
