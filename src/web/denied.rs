use topcoat::{
    Result,
    context::Cx,
    icon::{icon, iconify::iconify_icon},
    router::{error::redirect, page},
    view::view,
};

use crate::{
    components::button::{ButtonSize, ButtonVariant, button_variants},
    web::auth::current_user_id,
};

#[page]
pub(crate) async fn denied(cx: &Cx) -> Result {
    if current_user_id(cx).await.is_some() {
        return Err(redirect("/").into());
    }

    view! {
        <main class="flex min-h-dvh items-center justify-center p-4">
            <div
                class="flex flex-col items-center gap-4 rounded-lg border border-border bg-background p-8 text-center shadow-sm"
            >
                <span
                    class="flex size-12 items-center justify-center rounded-lg bg-primary text-primary-foreground"
                >
                    icon(data: iconify_icon!("feather:shield"))
                </span>
                <div class="space-y-1">
                    <h1 class="text-2xl font-semibold">"Not authorized"</h1>
                    <p class="text-sm text-muted-foreground">
                        "You must be a member of Spicy Boys."
                    </p>
                </div>
                <a
                    href="/oauth/discord"
                    class=(button_variants(
                        ButtonVariant::Primary,
                        ButtonSize::Md,
                    ))
                >
                    "Try again"
                </a>
            </div>
        </main>
    }
}
