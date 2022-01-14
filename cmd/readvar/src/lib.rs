// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! ## `humility readvar`
//!
//! `humility readvar` allows one to read a global static variable.
//! To list all such variables, use the `-l` option:
//!
//! ```console
//! % humility readvar -l
//! humility: MODULE             VARIABLE                       ADDR       SIZE
//! humility: kernel             CORE_PERIPHERALS               0x20000000 1
//! humility: kernel             CURRENT_TASK_PTR               0x20000018 4
//! humility: kernel             DEVICE_PERIPHERALS             0x20000001 1
//! humility: kernel             FAULT_NOTIFICATION             0x20000004 4
//! humility: kernel             IRQ_TABLE_BASE                 0x20000010 4
//! humility: kernel             IRQ_TABLE_SIZE                 0x20000014 4
//! humility: kernel             TASK_TABLE_BASE                0x20000008 4
//! humility: kernel             TASK_TABLE_SIZE                0x2000000c 4
//! humility: kernel             __EXCEPTIONS                   0x08000008 56
//! humility: kernel             __INTERRUPTS                   0x08000040 620
//! humility: kernel             __RESET_VECTOR                 0x08000004 4
//! humility: adt7420            TEMPS_BYMINUTE                 0x2000b848 17288
//! humility: adt7420            TEMPS_BYSECOND                 0x20008000 14408
//! ```
//!
//! To read a variable, specify it:
//!
//! ```console
//! % humility readvar CURRENT_TASK_PTR
//! humility: attached via ST-Link
//! CURRENT_TASK_PTR (0x20000018) = Some(NonNull<kern::task::Task> {
//!         pointer: 0x20000558 (*const kern::task::Task)
//!     })
//! ```
//!

use anyhow::{bail, Result};
use humility::core::Core;
use humility::hubris::*;
use humility_cmd::{Archive, Args, Attach, Command, Validate};
use structopt::clap::App;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "readvar", about = env!("CARGO_PKG_DESCRIPTION"))]
struct ReadvarArgs {
    /// values in decimal instead of hex
    #[structopt(long, short)]
    decimal: bool,
    /// list variables
    #[structopt(long, short)]
    list: bool,
    #[structopt(conflicts_with = "list")]
    variable: Option<String>,
}

fn readvar_dump(
    hubris: &HubrisArchive,
    core: &mut dyn Core,
    variable: &HubrisVariable,
    subargs: &ReadvarArgs,
) -> Result<()> {
    let mut buf: Vec<u8> = vec![];
    buf.resize_with(variable.size, Default::default);

    let _info = core.halt()?;
    core.read_8(variable.addr, buf.as_mut_slice())?;
    core.run()?;

    let hex = !subargs.decimal;

    let fmt = HubrisPrintFormat {
        newline: true,
        hex,
        ..HubrisPrintFormat::default()
    };
    let name = subargs.variable.as_ref().unwrap();
    let dumped = hubris.printfmt(&buf, variable.goff, &fmt)?;

    println!("{} (0x{:08x}) = {}", name, variable.addr, dumped);

    Ok(())
}

fn readvar(
    hubris: &mut HubrisArchive,
    core: &mut dyn Core,
    _args: &Args,
    subargs: &[String],
) -> Result<()> {
    let subargs = ReadvarArgs::from_iter_safe(subargs)?;

    if subargs.list {
        return hubris.list_variables();
    }

    let variables = match subargs.variable {
        Some(ref variable) => hubris.lookup_variables(variable)?,
        None => bail!("expected variable (use \"-l\" to list)"),
    };

    for v in variables {
        readvar_dump(hubris, core, v, &subargs)?;
    }

    Ok(())
}

pub fn init<'a, 'b>() -> (Command, App<'a, 'b>) {
    (
        Command::Attached {
            name: "readvar",
            archive: Archive::Required,
            attach: Attach::Any,
            validate: Validate::Match,
            run: readvar,
        },
        ReadvarArgs::clap(),
    )
}