//! Clinician commands.
//!
//! Each public command lives in its own submodule; shared types and the
//! legacy reqwest helpers used by the migration-pending commands live in
//! `data`, `output`, and `client`.

mod assign;
mod client;
mod data;
mod enable;
mod grant;
mod output;
mod prepare;
mod register;
mod show;
mod update;

#[cfg(test)]
mod testing;

pub use assign::assign;
pub use enable::{disable, enable};
pub use grant::grant;
pub use prepare::prepare;
pub use register::register;
pub use show::show;
pub use update::update;

use anyhow::Result;

use crate::cli::{CliniciansArgs, CliniciansCommands};
use crate::config::AppContext;
use crate::output::Output;

pub async fn dispatch(args: CliniciansArgs, ctx: &AppContext, out: &Output) -> Result<()> {
    match args.command {
        CliniciansCommands::Assign(a) => assign(ctx, &a.target, &a.team, out).await,
        CliniciansCommands::Grant(a) => grant(ctx, &a.target, &a.role, out).await,
        CliniciansCommands::Enable(a) => enable(ctx, &a.target, out).await,
        CliniciansCommands::Disable(a) => disable(ctx, &a.target, out).await,
        CliniciansCommands::Prepare(a) => prepare(ctx, &a.target, out).await,
        CliniciansCommands::Register(a) => {
            register(
                ctx,
                &a.email,
                &a.name,
                a.role.as_deref(),
                a.team.as_deref(),
                out,
            )
            .await
        }
        CliniciansCommands::Show(a) => show(ctx, &a.target, out).await,
        CliniciansCommands::Update(a) => {
            update(ctx, &a.target, &a.field, a.value.as_deref(), out).await
        }
    }
}
